use super::*;
use serde_json::json;

const GRAMMAR: &str = r#"{
    "formatVersion":1,"integer":{"kind":"integer","valueField":"value"},
    "conjunction":{"token":"and","kind":"conjunction","operandsField":"operands"},
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
        nodes: 0,
        engine: Engine::Compiled(generated::execute),
        primary_engine: Engine::Compiled(generated_primary::execute),
        record_engine: Engine::Compiled(generated_record::execute),
        conjunction_engine: Engine::Compiled(generated_conjunction::execute),
    };
    test(&mut Frame {
        parser: &mut parser,
        minimum: 0,
        depth: 0,
        left: None,
        right: None,
        operator: None,
        token_end: 0,
        consumed: false,
        binding: None,
        fuel: 4096,
        list_elements: Vec::new(),
        primary_state: Some(PrimaryState::IdentifierScanned(Some(3))),
        in_primary: true,
        record_state: None,
        index_state: None,
        selector_state: None,
        slice_upper: None,
        file: Default::default(),
    })
}

fn cross_guards(frame: &mut Frame<'_, '_>) {
    assert!(frame.has_conjunction().is_err());
    assert!(frame.conjunction().is_err());
    assert!(frame.record().is_err());
    assert!(frame.has_record().is_err());
    assert!(frame.advance_call().is_err());
    assert!(frame.parse_atom().is_err());
    assert!(frame.parse_integer().is_err());
    assert!(frame.scan_group().is_err());
    assert!(frame.primary().is_err());
    assert!(frame.lookup_operator().is_err());
    assert!(frame.scan_index().is_err());
    assert!(frame.scan_selector().is_err());
    assert!(frame.build_list().is_err());
}

#[test]
fn conjunction_stages_and_failed_final_charge_retain_buffer() -> TestResult {
    with_frame("and([1,])", |frame| {
        assert!(frame.has_conjunction()?);
        assert!(frame.advance_conjunction().is_err());
        frame.primary_state = Some(PrimaryState::ConjunctionSelected(3));
        cross_guards(frame);
        assert!(frame.open_conjunction_list().is_err());
        frame.advance_conjunction()?;
        cross_guards(frame);
        frame.open_conjunction_call()?;
        frame.whitespace()?;
        frame.open_conjunction_list()?;
        frame.whitespace()?;
        frame.probe_list_end()?;
        assert!(frame.has_list_element()?);
        cross_guards(frame);
        assert!(frame.close_conjunction_call().is_err());
        frame.parse_list_element(0)?;
        cross_guards(frame);
        assert!(frame.advance_list_tail().is_err());
        frame.append_list_element()?;
        cross_guards(frame);
        assert!(frame.append_list_element().is_err());
        frame.advance_list_tail()?;
        assert!(matches!(
            frame.primary_state,
            Some(PrimaryState::ConjunctionListReady)
        ));
        frame.whitespace()?;
        frame.probe_list_end()?;
        assert!(!frame.has_list_element()?);
        cross_guards(frame);
        assert!(frame.build_conjunction().is_err());
        frame.close_conjunction_call()?;
        cross_guards(frame);
        frame.parser.nodes = 64;
        assert!(frame.build_conjunction().is_err());
        assert_eq!(
            frame.list_elements,
            vec![json!({"kind":"integer","value":1})]
        );
        assert!(frame.left.is_none());
        frame.parser.nodes = 1;
        frame.build_conjunction()?;
        assert_eq!(frame.parser.nodes, 2);
        assert_eq!(
            frame.left,
            Some(json!({"kind":"conjunction","operands":[
                {"kind":"integer","value":1}
            ]}))
        );
        assert!(frame.list_elements.is_empty());
        assert!(frame.build_conjunction().is_err());
        Ok(())
    })
}

#[test]
fn conjunction_checked_spans_family_and_completion() -> TestResult {
    with_frame("and(λ)", |frame| {
        for end in [0, 5, 9] {
            frame.primary_state = Some(PrimaryState::ConjunctionSelected(end));
            assert!(frame.advance_conjunction().is_err());
            assert_eq!(frame.parser.offset, 0);
        }
        frame.primary_state = Some(PrimaryState::ListClosed);
        assert!(frame.close_conjunction_call().is_err());
        assert!(frame.build_conjunction().is_err());
        frame.primary_state = Some(PrimaryState::IdentifierScanned(Some(3)));
        frame.parser.conjunction_engine = Engine::Compiled(|_| Ok(()));
        assert!(frame.conjunction().is_err());
        assert!(frame.left.is_none());
        frame.primary_state = Some(PrimaryState::IdentifierScanned(Some(3)));
        frame.parser.conjunction_engine = Engine::Compiled(|f| {
            f.step()?;
            Ok(())
        });
        frame.fuel = 0;
        assert!(matches!(frame.conjunction(), Err(Error::ControlBudget)));
        assert!(frame.left.is_none());
        Ok(())
    })
}

#[test]
fn conjunction_empty_has_no_intermediate_list_node_or_fuel_refill() -> TestResult {
    with_frame("and([])", |frame| {
        frame.fuel = 4095;
        frame.conjunction()?;
        assert!(frame.fuel < 4095);
        assert_eq!(frame.parser.nodes, 1);
        assert_eq!(
            frame.left,
            Some(json!({"kind":"conjunction","operands":[]}))
        );
        Ok(())
    })
}
