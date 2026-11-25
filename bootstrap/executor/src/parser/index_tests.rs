use super::*;

#[test]
fn postfix_state_cursor_ownership_and_shared_fuel() -> Result<(), Box<dyn std::error::Error>> {
    let grammar: Grammar = serde_json::from_str(include_str!(concat!(
        env!("CUESTACEAN_BOOTSTRAP_JSON"),
        "/grammar.json"
    )))?;
    let mut parser = Parser {
        source: "[0]λ",
        grammar: &grammar,
        offset: 0,
        nodes: 0,
        engine: Engine::Compiled(generated::execute),
        primary_engine: Engine::Compiled(generated_primary::execute),
        record_engine: Engine::Compiled(generated_record::execute),
        conjunction_engine: Engine::Compiled(generated_conjunction::execute),
    };
    let mut frame = Frame {
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
        primary_state: Some(PrimaryState::Done),
        in_primary: false,
        record_state: None,
        list_elements: Vec::new(),
        index_state: None,
        selector_state: None,
        slice_upper: None,
        file: Default::default(),
    };
    assert!(frame.has_index().is_err());
    assert!(frame.parse_index(0).is_err());
    assert!(frame.close_index().is_err());
    for end in [0, 4, 6] {
        // No progress, split scalar, outside source.
        assert!(frame.advance_index_cursor(end).is_err());
        assert_eq!(frame.parser.offset, 0);
    }
    frame.in_primary = true;
    assert!(frame.scan_index().is_err());
    frame.in_primary = false;
    frame.scan_index()?;
    assert!(frame.has_index()?);
    assert!(frame.parse_index(0).is_err());
    frame.advance_index()?;
    assert!(frame.advance_index().is_err());
    assert!(frame.lookup_operator().is_err());
    assert!(frame.primary().is_err());
    assert!(frame.after_operand().is_err());
    assert!(frame.close_index().is_err());
    frame.fuel = 1;
    frame.step()?;
    frame.parse_index(0)?;
    assert_eq!(frame.fuel, 0); // A child gets its own frame, not a refill of this one.
    assert!(matches!(frame.step(), Err(Error::ControlBudget)));
    assert!(frame.parse_right().is_err());
    assert!(frame.parse_index(0).is_err());
    frame.close_index()?;
    assert_eq!(
        frame.left,
        Some(serde_json::json!({"kind":"index",
        "left":{"kind":"integer","value":7}, "right":{"kind":"integer","value":0}}))
    );
    assert!(frame.right.is_none());
    assert_eq!(frame.parser.nodes, 2);
    assert!(frame.close_index().is_err());
    Ok(())
}
