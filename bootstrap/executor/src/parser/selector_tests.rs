use super::*;

const GRAMMAR: &str = r#"{
    "formatVersion":1,"integer":{"kind":"integer","valueField":"value"},
    "index":{"kind":"index","leftField":"left","rightField":"right"},
    "selector":{"kind":"select","operandField":"operand","labelField":"label"},
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
        left: Some(serde_json::json!({"kind":"integer","value":7})),
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
    })
}

#[test]
fn selector_stages_cross_phase_progress_and_fuel() -> TestResult {
    with_frame(". // trivia\n\"λ\\u0061\"", |frame| {
        assert!(frame.has_postfix().is_err());
        assert!(frame.scan_selector().is_err());
        assert!(frame.advance_selector().is_err());
        assert!(frame.selector_whitespace().is_err());
        assert!(frame.scan_selector_string().is_err());
        assert!(frame.has_selector_string().is_err());
        assert!(frame.parse_selector_string().is_err());
        assert!(frame.parse_selector_identifier().is_err());
        assert!(frame.build_selector().is_err());
        frame.scan_index()?;
        assert!(!frame.has_index()?);
        frame.in_primary = true;
        assert!(frame.scan_selector().is_err());
        frame.in_primary = false;
        frame.scan_selector()?;
        assert!(frame.has_postfix()?);
        assert!(frame.selector_pending());
        assert!(frame.scan_selector().is_err());
        assert!(frame.scan_index().is_err());
        assert!(frame.advance_index().is_err());
        assert!(frame.primary().is_err());
        assert!(frame.lookup_operator().is_err());
        assert!(frame.after_operand().is_err());
        assert!(frame.set_binding_power(1).is_err());
        assert!(frame.parse_right().is_err());
        assert!(frame.build_binary().is_err());
        for end in [0, 14, 100] {
            // No progress, split UTF-8 scalar, outside source.
            assert!(frame.selector_span(end).is_err());
            assert_eq!(frame.parser.offset, 0);
        }
        frame.advance_selector()?;
        assert_eq!(frame.parser.offset, 1);
        assert!(frame.advance_selector().is_err());
        assert!(frame.scan_selector_string().is_err());
        assert!(frame.whitespace().is_err()); // Existing primary-only action stays private.
        frame.fuel = 1;
        frame.step()?;
        frame.selector_whitespace()?;
        assert_eq!(frame.fuel, 0);
        assert!(matches!(frame.step(), Err(Error::ControlBudget)));
        assert!(frame.selector_whitespace().is_err());
        frame.scan_selector_string()?;
        assert!(frame.has_selector_string()?);
        assert!(frame.parse_selector_identifier().is_err());
        assert!(frame.build_selector().is_err());
        frame.parse_selector_string()?;
        assert!(frame.parse_selector_string().is_err());
        assert_eq!(frame.parser.nodes, 1); // Labels are data, not AST children.
        frame.build_selector()?;
        assert_eq!(
            frame.left,
            Some(serde_json::json!({
                "kind":"select", "operand":{"kind":"integer","value":7}, "label":"λa"
            }))
        );
        assert_eq!(frame.parser.nodes, 2);
        assert!(frame.index_state.is_none());
        assert!(frame.selector_state.is_none());
        assert!(frame.build_selector().is_err());
        frame.after_operand()?;
        frame.scan_index()?;
        frame.scan_selector()?;
        assert!(!frame.has_postfix()?);
        assert!(!frame.selector_pending());
        Ok(())
    })
}

#[test]
fn index_candidate_cannot_probe_selector() -> TestResult {
    with_frame("[0]", |frame| {
        frame.scan_index()?;
        assert!(frame.has_postfix()?);
        assert!(frame.scan_selector().is_err());
        frame.advance_index()?;
        assert!(frame.has_postfix().is_err());
        assert!(frame.scan_selector().is_err());
        Ok(())
    })
}

#[test]
fn selector_identifier_and_empty_string_are_literal_data() -> TestResult {
    with_frame(".true", |frame| {
        frame.scan_index()?;
        frame.scan_selector()?;
        frame.advance_selector()?;
        frame.selector_whitespace()?;
        frame.scan_selector_string()?;
        assert!(!frame.has_selector_string()?);
        assert!(frame.parse_selector_string().is_err());
        frame.parse_selector_identifier()?;
        frame.build_selector()?;
        assert_eq!(
            frame.left,
            Some(serde_json::json!({
                "kind":"select", "operand":{"kind":"integer","value":7}, "label":"true"
            }))
        );
        Ok(())
    })?;
    with_frame(".\"\"", |frame| {
        frame.scan_index()?;
        frame.scan_selector()?;
        frame.advance_selector()?;
        frame.selector_whitespace()?;
        frame.scan_selector_string()?;
        frame.parse_selector_string()?;
        frame.parser.nodes = 63;
        frame.build_selector()?;
        assert_eq!(frame.parser.nodes, 64);
        assert_eq!(
            frame.left,
            Some(serde_json::json!({
                "kind":"select", "operand":{"kind":"integer","value":7}, "label":""
            }))
        );
        Ok(())
    })
}

#[test]
fn failed_selector_node_charge_preserves_pending_label() -> TestResult {
    with_frame(".x", |frame| {
        frame.scan_index()?;
        frame.scan_selector()?;
        frame.advance_selector()?;
        frame.selector_whitespace()?;
        frame.scan_selector_string()?;
        frame.parse_selector_identifier()?;
        frame.parser.nodes = 64;
        assert!(matches!(frame.build_selector(), Err(Error::Source { .. })));
        assert!(frame.selector_pending());
        assert!(matches!(&frame.selector_state, Some(SelectorState::Label(label)) if label == "x"));
        assert_eq!(
            frame.left,
            Some(serde_json::json!({"kind":"integer","value":7}))
        );
        Ok(())
    })
}

#[test]
fn selector_metadata_is_validated_even_without_syntax() -> TestResult {
    for selector in [
        serde_json::json!({"kind":"", "operandField":"operand", "labelField":"label"}),
        serde_json::json!({"kind":"s", "operandField":"", "labelField":"label"}),
        serde_json::json!({"kind":"s", "operandField":"operand", "labelField":""}),
        serde_json::json!({"kind":"s", "operandField":"same", "labelField":"same"}),
        serde_json::json!({"kind":"s", "operandField":"kind", "labelField":"label"}),
        serde_json::json!({"kind":"s", "operandField":"operand", "labelField":"kind"}),
    ] {
        let mut grammar: Value = serde_json::from_str(GRAMMAR)?;
        grammar["selector"] = selector;
        assert!(matches!(
            parse("0", &grammar.to_string()),
            Err(Error::Grammar { .. })
        ));
    }
    let mut grammar: Value = serde_json::from_str(GRAMMAR)?;
    grammar
        .as_object_mut()
        .ok_or("grammar object")?
        .remove("selector");
    assert!(parse("0.x", &grammar.to_string()).is_err());
    Ok(())
}

#[test]
fn unfinished_selector_cannot_escape_driver_return() -> TestResult {
    fn pending(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        frame.primary()?;
        frame.after_operand()?;
        frame.scan_index()?;
        frame.scan_selector()?;
        Ok(())
    }
    assert!(matches!(
        parse_with_driver("0.x", GRAMMAR, pending),
        Err(Error::ControlState {
            reason: "unfinished selector at frame return"
        })
    ));
    Ok(())
}
