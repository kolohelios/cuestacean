//! Same-source compiler evidence; full-library verification, not native construction.
#[cfg(feature = "compiled-lowering")]
pub use bootstrap_executor::compiled_pratt_lowering;
pub use bootstrap_executor::lowering;
#[path = "../src/pratt/compiler.rs"]
mod compiler;
#[path = "../src/unification/kind.rs"]
mod unification;

fn control(target: &str, body: &str) -> String {
    format!(r#"{{"formatVersion":1,{target}"body":{body}}}"#)
}

struct Metadata(serde_json::Value);
impl lowering::Driver for Metadata {
    fn render(
        &self,
        _: &mut lowering::Context,
        node: &serde_json::Value,
        _: usize,
    ) -> Result<String, lowering::Error> {
        assert_eq!(node, &self.0);
        Ok("checked".to_owned())
    }
}

#[test]
fn complete_metadata_for_five_targets_and_authoritative_kinds() {
    for (target, body, expected) in [
        (
            "",
            r#"[{"op":"primary"},{"op":"scanSelector"},{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"return"}]},{"op":"while","condition":{"op":"boolean","value":false},"body":[{"op":"return"}]},{"op":"if","condition":{"op":"boolean","value":true},"then":[{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"return"}],"else":[{"op":"return"}]}],"else":[{"op":"return"}]}]"#,
            r#"{"op":"program","target":"parser","tail":"return","body":[{"op":"action","action":"primary"},{"op":"selectorAction","action":"scanSelector"},{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"return","position":"early"}],"else":[]},{"op":"while","condition":{"op":"boolean","value":false},"body":[{"op":"return","position":"early"}]},{"op":"if","condition":{"op":"boolean","value":true},"then":[{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"return","position":"tail"}],"else":[{"op":"return","position":"tail"}]}],"else":[{"op":"return","position":"tail"}]}]}"#,
        ),
        (
            r#""target":"list","#,
            r#"[{"op":"if","condition":{"op":"itemKindIs","kind":"integer"},"then":[{"op":"appendItem"}]}]"#,
            r#"{"op":"program","target":"list","tail":"value","body":[{"op":"if","condition":{"op":"itemKindIs","kind":"integer"},"then":[{"op":"action","action":"appendItem"}],"else":[]}]}"#,
        ),
        (
            r#""target":"record","#,
            r#"[{"op":"finishRecord"},{"op":"return"}]"#,
            r#"{"op":"program","target":"record","tail":"return","body":[{"op":"action","action":"finishRecord"},{"op":"return","position":"tail"}]}"#,
        ),
        (
            r#""target":"conjunction","#,
            r#"[{"op":"seedTop"}]"#,
            r#"{"op":"program","target":"conjunction","tail":"value","body":[{"op":"action","action":"seedTop"}]}"#,
        ),
        (
            r#""target":"slice","#,
            r#"[{"op":"if","condition":{"op":"sliceKindIs","kind":"list"},"then":[{"op":"defaultSliceLower","value":-9223372036854775808}]}]"#,
            r#"{"op":"program","target":"slice","tail":"value","body":[{"op":"if","condition":{"op":"sliceKindIs","kind":"list"},"then":[{"op":"defaultSliceLower","value":-9223372036854775808}],"else":[]}]}"#,
        ),
    ] {
        let program = compiler::parse(&control(target, body)).expect("valid control");
        let driver = Metadata(serde_json::from_str(expected).expect("handwritten metadata"));
        assert_eq!(
            compiler::lower(program, &driver).expect("real context"),
            "checked"
        );
    }
    use unification::Kind;
    for (kind, wire) in [
        (Kind::Null, "null"),
        (Kind::Top, "top"),
        (Kind::Integer, "integer"),
        (Kind::Int, "int"),
        (Kind::Bounds, "bounds"),
        (Kind::Boolean, "boolean"),
        (Kind::Bool, "bool"),
        (Kind::Text, "text"),
        (Kind::String, "string"),
        (Kind::List, "list"),
        (Kind::Record, "record"),
    ] {
        assert_eq!(
            serde_json::to_value(kind).expect("Kind serialization"),
            wire
        );
        for (target, op) in [("list", "itemKindIs"), ("slice", "sliceKindIs")] {
            let input = format!(
                r#"{{"formatVersion":1,"target":"{target}","body":[{{"op":"if","condition":{{"op":"{op}","kind":"{wire}"}},"then":[]}}]}}"#
            );
            let expected = serde_json::json!({"op":"program","target":target,"tail":"value","body":[{"op":"if","condition":{"op":op,"kind":wire},"then":[],"else":[]}]});
            assert_eq!(
                compiler::lower(
                    compiler::parse(&input).expect("Kind wire"),
                    &Metadata(expected)
                )
                .expect("Kind metadata"),
                "checked"
            );
        }
    }
}

#[test]
fn typed_rejections_precede_invalid_policy() {
    for (target, body) in [
        (
            r#""target":"list","#,
            r#"[{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"primary"}]}]"#,
        ),
        (
            "",
            r#"[{"op":"while","condition":{"op":"and","left":{"op":"boolean","value":false},"right":{"op":"hasItem"}},"body":[]}]"#,
        ),
        (
            r#""target":"slice","#,
            r#"[{"op":"if","condition":{"op":"boolean","value":false},"then":[],"else":[{"op":"appendItem"}]}]"#,
        ),
        (r#""target":"conjunction","#, r#"[{"op":"finishRecord"}]"#),
        (r#""target":"record","#, r#"[{"op":"seedTop"}]"#),
        ("", r#"[{"op":"return"},{"op":"primary"}]"#),
        (
            "",
            r#"[{"op":"while","condition":{"op":"boolean","value":false},"body":[{"op":"return"},{"op":"primary"}]}]"#,
        ),
        (
            "",
            r#"[{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":"return"},{"op":"primary"}]}]"#,
        ),
        (
            "",
            r#"[{"op":"if","condition":{"op":"boolean","value":true},"then":[{"op":"return"}],"else":[{"op":"return"}]},{"op":"primary"}]"#,
        ),
    ] {
        assert!(
            matches!(
                compiler::generate_with_policy(&control(target, body), "not JSON"),
                Err(compiler::Error::Program { .. })
            ),
            "{target}{body}"
        );
    }
    for input in [
        r#"{"formatVersion":1,"formatVersion":1,"body":[]}"#,
        r#"{"formatVersion":1,"body":[{"op":"primary","op":"return"}]}"#,
        r#"{"formatVersion":1,"target":"List","body":[]}"#,
        r#"{"formatVersion":1,"target":0,"body":[]}"#,
        r#"{"formatVersion":1,"body":[{"op":"parseGroup","minimum":{"op":"integer","value":256}}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"parseGroup","minimum":{"op":"integer","value":-1}}]}"#,
        r#"{"formatVersion":1,"target":"slice","body":[{"op":"defaultSliceLower","value":9223372036854775808}]}"#,
        r#"{"formatVersion":1,"target":"slice","body":[{"op":"defaultSliceLower","value":-9223372036854775809}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"action","action":"primary"}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"selectorAction","action":"scanSelector"}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"parseMinimum","minimum":{"op":"integer","value":0}}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"primary","action":"primary"}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"return","position":"tail"}]}"#,
        r#"{"formatVersion":1,"tail":"return","body":[]}"#,
    ] {
        assert!(
            matches!(
                compiler::generate_with_policy(input, "not JSON"),
                Err(compiler::Error::Json { .. })
            ),
            "{input}"
        );
    }
}

#[test]
fn real_default_and_reference_complete_rust() {
    let policy = include_str!(concat!(
        env!("CUESTACEAN_BOOTSTRAP_JSON"),
        "/pratt-lowering.json"
    ));
    for (body, expected) in [
        (
            r#"[{"op":"primary"}]"#,
            "// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::parser::{Error, Frame};\npub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {\n_frame.step()?;\n_frame.primary()?;\nOk(())\n}\n",
        ),
        (
            r#"[{"op":"return"}]"#,
            "// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::parser::{Error, Frame};\npub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {\n_frame.step()?;\nOk(())\n}\n",
        ),
    ] {
        let input = control("", body);
        assert_eq!(
            compiler::generate(&input).expect("default backend"),
            expected
        );
        assert_eq!(
            compiler::generate_with_policy(&input, policy).expect("reference backend"),
            expected
        );
    }
}

#[test]
fn custom_policy_reads_target_tail_body_and_recovers() {
    let policy = r#"{"formatVersion":1,"cases":{"program":{"op":"concat","parts":[{"op":"string","field":"target"},{"op":"string","field":"tail"},{"op":"list","field":"body"}]},"action":{"op":"string","field":"action"},"selectorAction":{"op":"string","field":"action"},"return":{"op":"string","field":"position"}}}"#;
    let valid = control("", r#"[{"op":"primary"}]"#);
    assert!(matches!(
        compiler::generate_with_policy(&valid, "not JSON"),
        Err(compiler::Error::Lowering { .. })
    ));
    let missing_case = control(
        "",
        r#"[{"op":"while","condition":{"op":"boolean","value":false},"body":[]}]"#,
    );
    assert!(matches!(
        compiler::generate_with_policy(&missing_case, policy),
        Err(compiler::Error::Lowering { .. })
    ));
    for (target, body, expected) in [
        (
            "",
            r#"[{"op":"primary"},{"op":"scanSelector"}]"#,
            r#""parser""value""primary""scanSelector""#,
        ),
        (
            r#""target":"record","#,
            r#"[{"op":"finishRecord"},{"op":"return"}]"#,
            r#""record""return""finishRecord""tail""#,
        ),
    ] {
        assert_eq!(
            compiler::generate_with_policy(&control(target, body), policy)
                .expect("custom metadata after failure"),
            expected
        );
    }
}
