use super::*;

const GRAMMAR: &str = program!("grammar");

fn run(driver: Driver) -> Result<Value, Error> {
    let grammar: Grammar = serde_json::from_str(GRAMMAR).context(JsonSnafu)?;
    let mut parser = Parser {
        source: "a:1\nb:2",
        grammar: &grammar,
        offset: 0,
        nodes: 0,
        engine: Engine::Compiled(expression::execute),
        primary_engine: Engine::Compiled(generated_primary::execute),
        record_engine: Engine::Compiled(generated_record::execute),
        conjunction_engine: Engine::Compiled(generated_conjunction::execute),
    };
    parser.expression_control(0, 0, false, Engine::Compiled(driver))
}

#[test]
fn checked_file_stages_and_completion() -> Result<(), Error> {
    fn stages(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        assert!(frame.start_file().is_err());
        assert!(frame.build_file().is_err());
        assert!(frame.file_boundary().is_err());
        frame.probe_file()?;
        assert_eq!(frame.parser.offset, 0);
        assert!(frame.probe_file().is_err());
        assert!(frame.has_file_field()?);
        frame.start_file()?;
        assert!(frame.start_file().is_err());
        assert!(frame.parse_file_value().is_err());
        frame.parse_file_label()?;
        assert!(frame.parse_file_label().is_err());
        frame.parse_file_colon()?;
        frame.parse_file_value()?;
        assert!(frame.parse_file_value().is_err());
        assert_eq!(frame.parser.offset, 3);
        frame.append_file_field()?;
        assert!(frame.append_file_field().is_err());
        frame.advance_file_tail()?;
        assert!(frame.advance_file_tail().is_err());
        assert_eq!(frame.parser.offset, 4);
        frame.parse_file_label()?;
        frame.parse_file_colon()?;
        frame.parse_file_value()?;
        frame.append_file_field()?;
        frame.advance_file_tail()?;
        assert!(!frame.has_file_field()?);
        frame.build_file()?;
        assert!(frame.build_file().is_err());
        assert_eq!(frame.parser.nodes, 3);
        Ok(())
    }
    run(stages)?;
    assert!(matches!(
        run(|frame| frame.probe_file()),
        Err(Error::ControlState { .. })
    ));
    fn budget(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        frame.fuel = 1;
        primary::execute(frame)
    }
    assert!(matches!(run(budget), Err(Error::ControlBudget)));
    Ok(())
}

#[test]
fn fields_only_preserves_origin_and_existing_entries() -> Result<(), Error> {
    let source = "// heading\r\n age: int & >=0\r\ntrue: null\n";
    let expected = serde_json::json!({"formatVersion":1,"entry":{
    "kind":"recordExpression","fields":[
        {"name":"age","value":{"kind":"unify","left":{"kind":"intType"},
            "right":{"kind":"lowerBound","value":0}}},
        {"name":"true","value":{"kind":"null"}}
    ]}});
    assert_eq!(parse_fields_file(source, GRAMMAR)?, expected);
    assert_eq!(parse_file(source, GRAMMAR)?, expected);
    assert_eq!(parse_file_interpreted(source, GRAMMAR)?, expected);
    // These intentionally share an AST with ordinary declarations in old mode.
    // Reject before expression parsing in fields-only mode, even when grouped.
    for source in [
        "close({\"x\":int})",
        "(close({\"x\":int}))",
        "// comment\n42",
        "1+2",
    ] {
        assert!(parse_file(source, GRAMMAR).is_ok(), "{source}");
        assert!(parse_file_interpreted(source, GRAMMAR).is_ok(), "{source}");
        assert!(
            matches!(
                parse_fields_file(source, GRAMMAR),
                Err(Error::Source { .. })
            ),
            "{source}"
        );
    }
    for source in [
        "",
        "// only\n",
        "x:1\nx:2",
        "x!: int",
        "x?: int",
        "x:1, y:2",
        "x: unknown",
    ] {
        assert!(parse_fields_file(source, GRAMMAR).is_err(), "{source}");
    }
    // Shared node count includes the file record. No budget is reset per field.
    let wide = (0..63).map(|i| format!("x{i}:1\n")).collect::<String>();
    assert!(parse_fields_file(&wide, GRAMMAR).is_ok());
    assert!(parse_fields_file(&format!("{wide}last:1\n"), GRAMMAR).is_err());
    Ok(())
}

#[test]
fn fields_only_positive_probe_never_enters_expression_fallback() -> Result<(), Error> {
    fn guarded(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        frame.file.require_fields = true;
        frame.probe_file()?;
        assert!(frame.has_file_field()?);
        assert!(frame.parse_file_expression().is_err());
        assert_eq!(frame.parser.offset, 0);
        Err(Error::ControlState {
            reason: "test stops after probe",
        })
    }
    assert!(matches!(run(guarded), Err(Error::ControlState { .. })));
    Ok(())
}

#[test]
fn vet_list_minimum_propagates_into_nested_tails() -> Result<(), Error> {
    let source = "a: [[...int & >=0]]";
    let grammar = program!("vet-grammar");
    for reference in [false, true] {
        let normal = if reference {
            parse_vet_fields_file_interpreted(source, grammar)
        } else {
            parse_vet_fields_file(source, grammar)
        };
        normal?;
        assert!(matches!(
            parse_vet_fields_file_high_minimum(source, grammar, reference),
            Err(Error::Source { offset: 12, reason })
                if reason == "expected closing bracket after list tail"
        ));
    }
    Ok(())
}

#[test]
fn file_overlay_is_explicit_and_parser_family_only() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{"formatVersion":1,"body":[{"op":"if","condition":{"op":"fileBoundary"},"then":[{"op":"return"}]}]}"#;
    assert!(crate::pratt::generate(input).is_err());
    let expected = "// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::parser::{Error, Frame};\npub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {\n_frame.step()?;\nif _frame.file_boundary()? {\n_frame.step()?;\nreturn Ok(());\n}\nOk(())\n}\n";
    assert_eq!(
        crate::pratt::generate_with_policy(input, program!("file-pratt-lowering"))?,
        expected
    );
    for target in ["list", "record", "conjunction", "slice"] {
        let input = input.replace("\"body\":", &format!("\"target\":\"{target}\",\"body\":"));
        assert!(
            crate::pratt::generate_with_policy(&input, program!("file-pratt-lowering")).is_err()
        );
    }
    Ok(())
}
