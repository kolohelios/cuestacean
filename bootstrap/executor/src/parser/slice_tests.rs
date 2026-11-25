use super::*;
use serde_json::json;

const GRAMMAR: &str = r#"{
    "formatVersion":1,"integer":{"kind":"integer","valueField":"value"},
    "index":{"kind":"index","leftField":"left","rightField":"right"},
    "slice":{"kind":"range","operandField":"base","lowerField":"lo","upperField":"hi"},
    "group":{"open":"(","close":")"},
    "operators":[{"token":"+","kind":"add","leftField":"left",
        "rightField":"right","precedence":10,"associativity":"left"}]
}"#;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn with_frame(source: &str, test: impl FnOnce(&mut Frame<'_, '_>) -> TestResult) -> TestResult {
    let grammar: Grammar = serde_json::from_str(GRAMMAR)?;
    grammar.validate()?;
    let mut parser = Parser {
        source,
        grammar: &grammar,
        offset: 0,
        nodes: 1,
        engine: Engine::Compiled(generated::execute),
        primary_engine: Engine::Compiled(generated_primary::execute),
        record_engine: Engine::Compiled(generated_record::execute),
        conjunction_engine: Engine::Compiled(generated_conjunction::execute),
    };
    test(&mut Frame {
        parser: &mut parser,
        minimum: 0,
        depth: 0,
        left: Some(json!({"kind":"integer","value":7})),
        right: None,
        operator: None,
        token_end: 0,
        consumed: false,
        binding: None,
        fuel: 4096,
        list_elements: Vec::new(),
        primary_state: Some(PrimaryState::Done),
        in_primary: false,
        record_state: None,
        index_state: None,
        selector_state: None,
        slice_upper: None,
        file: Default::default(),
    })
}

fn lower(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
    frame.scan_index()?;
    frame.advance_index()?;
    frame.parse_index(0)
}

fn pending_rejects_other_phases(frame: &mut Frame<'_, '_>) {
    assert!(frame.primary().is_err());
    assert!(frame.scan_index().is_err());
    assert!(frame.scan_selector().is_err());
    assert!(frame.has_postfix().is_err());
    assert!(frame.after_operand().is_err());
    assert!(frame.lookup_operator().is_err());
    assert!(frame.set_binding_power(1).is_err());
    assert!(frame.parse_right().is_err());
    assert!(frame.build_binary().is_err());
}

#[test]
fn slice_stages_and_complete_tree() -> TestResult {
    with_frame("[0: // endpoint\n 1+2]", |frame| {
        assert!(frame.scan_slice().is_err());
        assert!(frame.has_slice().is_err());
        assert!(frame.advance_slice().is_err());
        assert!(frame.parse_slice_upper(0).is_err());
        assert!(frame.close_slice().is_err());
        lower(frame)?;
        frame.in_primary = true;
        assert!(frame.scan_slice().is_err());
        frame.in_primary = false;
        frame.scan_slice()?;
        assert!(frame.has_slice()?);
        assert!(frame.scan_slice().is_err());
        assert!(frame.close_index().is_err());
        assert!(frame.parse_slice_upper(0).is_err());
        pending_rejects_other_phases(frame);
        frame.advance_slice()?;
        assert!(frame.advance_slice().is_err());
        assert!(frame.close_slice().is_err());
        pending_rejects_other_phases(frame);
        frame.fuel = 1;
        frame.step()?;
        frame.parse_slice_upper(0)?;
        assert_eq!(frame.fuel, 0);
        assert!(matches!(frame.step(), Err(Error::ControlBudget)));
        assert!(frame.parse_slice_upper(0).is_err());
        assert!(frame.close_index().is_err());
        pending_rejects_other_phases(frame);
        frame.close_slice()?;
        assert_eq!(
            frame.left,
            Some(json!({"kind":"range",
            "base":{"kind":"integer","value":7},"lo":{"kind":"integer","value":0},
            "hi":{"kind":"add","left":{"kind":"integer","value":1},
                "right":{"kind":"integer","value":2}}}))
        );
        assert!(frame.right.is_none() && frame.slice_upper.is_none());
        assert!(frame.index_state.is_none() && frame.selector_state.is_none());
        assert_eq!(frame.parser.nodes, 6);
        assert!(frame.close_slice().is_err());
        Ok(())
    })
}

#[test]
fn negative_probe_and_failed_charge_keep_operands() -> TestResult {
    with_frame("[0]", |frame| {
        lower(frame)?;
        frame.scan_slice()?;
        assert!(!frame.has_slice()?);
        assert!(frame.advance_slice().is_err());
        frame.close_index()?;
        Ok(())
    })?;
    with_frame("[0:1]", |frame| {
        lower(frame)?;
        frame.scan_slice()?;
        frame.advance_slice()?;
        frame.parse_slice_upper(0)?;
        let operands = (
            frame.left.clone(),
            frame.right.clone(),
            frame.slice_upper.clone(),
        );
        frame.parser.nodes = 64;
        assert!(frame.close_slice().is_err());
        assert_eq!(
            (
                frame.left.clone(),
                frame.right.clone(),
                frame.slice_upper.clone()
            ),
            operands
        );
        assert!(matches!(frame.index_state, Some(IndexState::SliceParsed)));
        pending_rejects_other_phases(frame);
        Ok(())
    })
}

#[test]
fn upper_minimum_and_depth_are_not_native_policy() -> TestResult {
    with_frame("[0:1+2]", |frame| {
        lower(frame)?;
        frame.scan_slice()?;
        frame.advance_slice()?;
        frame.parse_slice_upper(100)?;
        assert_eq!(
            frame.slice_upper.as_deref(),
            Some(&json!({"kind":"integer","value":1}))
        );
        assert!(frame.close_slice().is_err());
        Ok(())
    })?;
    with_frame("[0:1]", |frame| {
        lower(frame)?;
        frame.scan_slice()?;
        frame.advance_slice()?;
        frame.depth = 32;
        assert!(matches!(
            frame.parse_slice_upper(0),
            Err(Error::Source { .. })
        ));
        assert!(frame.slice_upper.is_none());
        assert!(frame.left.is_some() && frame.right.is_some());
        assert!(matches!(frame.index_state, Some(IndexState::SliceAdvanced)));
        Ok(())
    })
}

#[test]
fn unfinished_slice_cannot_escape_frame() -> TestResult {
    fn unfinished(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        frame.primary()?;
        frame.scan_index()?;
        frame.advance_index()?;
        frame.parse_index(0)?;
        frame.scan_slice()?;
        frame.advance_slice()?;
        // Child expressions need to finish normally with this same selected driver.
        Ok(())
    }
    // The inner lower has no bracket; use a guarded driver for all recursion.
    fn driver(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        if frame.depth == 0 {
            unfinished(frame)
        } else {
            generated::execute(frame)
        }
    }
    assert!(matches!(
        parse_with_driver("7[0:1]", GRAMMAR, driver),
        Err(Error::ControlState { .. })
    ));
    Ok(())
}

#[test]
fn slice_metadata_is_eager_and_optional() -> TestResult {
    let original: Value = serde_json::from_str(GRAMMAR)?;
    for field in ["operandField", "lowerField", "upperField"] {
        for invalid in ["", "kind"] {
            let mut metadata = original.clone();
            metadata["slice"][field] = json!(invalid);
            assert!(
                serde_json::from_value::<Grammar>(metadata)?
                    .validate()
                    .is_err()
            );
        }
    }
    for (a, b) in [
        ("operandField", "lowerField"),
        ("operandField", "upperField"),
        ("lowerField", "upperField"),
    ] {
        let mut metadata = original.clone();
        metadata["slice"][a] = metadata["slice"][b].clone();
        assert!(
            serde_json::from_value::<Grammar>(metadata)?
                .validate()
                .is_err()
        );
    }
    let mut metadata = original.clone();
    metadata["slice"]["kind"] = json!("");
    assert!(
        serde_json::from_value::<Grammar>(metadata)?
            .validate()
            .is_err()
    );
    let mut metadata = original.clone();
    metadata["index"] = Value::Null;
    assert!(
        serde_json::from_value::<Grammar>(metadata.clone())?
            .validate()
            .is_err()
    );
    metadata["slice"] = Value::Null;
    serde_json::from_value::<Grammar>(metadata)?.validate()?;
    Ok(())
}
