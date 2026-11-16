//! Typed, CUE-authored instruction lowering policy.
use std::{collections::BTreeMap, fmt, fmt::Write as _};

use serde::{
    Deserialize, Deserializer,
    de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid lowering policy JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("lowering: {reason}"))]
    Invalid { reason: String },
}

fn invalid(reason: impl Into<String>) -> Error {
    Error::Invalid {
        reason: reason.into(),
    }
}

pub struct Policy {
    cases: BTreeMap<String, Expr>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPolicy {
    format_version: u32,
    #[serde(deserialize_with = "unique_cases")]
    cases: BTreeMap<String, Expr>,
}

// Unlike a struct, a map normally accepts duplicate keys during deserialization.
fn unique_cases<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Expr>, D::Error> {
    struct Cases;
    impl<'de> Visitor<'de> for Cases {
        type Value = BTreeMap<String, Expr>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map of unique lowering cases")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut cases = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, Expr>()? {
                if cases.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!("duplicate case {key:?}")));
                }
            }
            Ok(cases)
        }
    }
    deserializer.deserialize_map(Cases)
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Expr {
    Path { value: String },
    Call { path: String, args: Vec<Expr> },
    Ok { value: Box<Expr> },
    Group { value: Box<Expr> },
    Try { value: Box<Expr> },
    Lower { field: String },
    String { field: String },
    Boolean { field: String },
}

// Count original values, including overwritten duplicate keys, before typed decode.
struct JsonCheck<'a> {
    depth: usize,
    nodes: &'a mut usize,
}

impl<'de> DeserializeSeed<'de> for JsonCheck<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        *self.nodes += 1;
        if self.depth > 32 || *self.nodes > 512 {
            return Err(serde::de::Error::custom(
                "policy exceeds 32 JSON levels or 512 JSON nodes",
            ));
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for JsonCheck<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while seq
            .next_element_seed(JsonCheck {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while map.next_key::<IgnoredAny>()?.is_some() {
            map.next_value_seed(JsonCheck {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?;
        }
        Ok(())
    }
}

fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 128
        && path.split("::").all(|part| {
            let mut bytes = part.bytes();
            matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
                && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        })
}

impl Expr {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Path { value } if !valid_path(value) => Err(invalid("invalid Rust path")),
            Self::Call { path, args } => {
                if !valid_path(path) {
                    return Err(invalid("invalid Rust path"));
                }
                for arg in args {
                    arg.validate()?;
                }
                Ok(())
            }
            Self::Ok { value } | Self::Group { value } | Self::Try { value } => value.validate(),
            Self::Lower { field } | Self::String { field } | Self::Boolean { field }
                if field.is_empty() || field.len() > 128 =>
            {
                Err(invalid("selector must contain 1..=128 bytes"))
            }
            _ => Ok(()),
        }
    }
}

impl Policy {
    /// Decode the original policy text and validate every template before use.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        JsonCheck {
            depth: 1,
            nodes: &mut 0,
        }
        .deserialize(&mut deserializer)
        .context(JsonSnafu)?;
        deserializer.end().context(JsonSnafu)?;
        let policy: RawPolicy = serde_json::from_str(input).context(JsonSnafu)?;
        if policy.format_version != 1 {
            return Err(invalid("formatVersion 1 required"));
        }
        if policy.cases.is_empty() || policy.cases.len() > 64 {
            return Err(invalid("policy must contain 1..=64 cases"));
        }
        for (key, expr) in &policy.cases {
            if key.is_empty() || key.len() > 64 {
                return Err(invalid("case key must contain 1..=64 bytes"));
            }
            expr.validate()?;
        }
        Ok(Self {
            cases: policy.cases,
        })
    }

    /// Lower one instruction with shared recursion, work and output budgets.
    pub fn lower(&self, instruction: &Value) -> Result<String, Error> {
        let mut budget = Budget {
            steps: 0,
            bytes: 0,
            frames: 0,
        };
        self.dispatch(instruction, 1, &mut budget)
    }

    fn dispatch(
        &self,
        instruction: &Value,
        depth: usize,
        budget: &mut Budget,
    ) -> Result<String, Error> {
        budget.with_frame(|budget| {
            budget.step()?;
            if depth > 64 {
                return Err(invalid("instruction depth exceeds 64"));
            }
            let op = instruction
                .get("op")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("instruction requires string op"))?;
            let expr = self
                .cases
                .get(op)
                .ok_or_else(|| invalid(format!("missing lowering case for {op:?}")))?;
            self.render(expr, instruction, depth, budget)
        })
    }

    fn render(
        &self,
        expr: &Expr,
        node: &Value,
        depth: usize,
        budget: &mut Budget,
    ) -> Result<String, Error> {
        budget.with_frame(|budget| {
            budget.step()?;
            let mut out = String::new();
            match expr {
                Expr::Path { value } => budget.append(&mut out, value)?,
                Expr::Call { path, args } => {
                    budget.append(&mut out, path)?;
                    budget.append(&mut out, "(")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i != 0 {
                            budget.append(&mut out, ", ")?;
                        }
                        let rendered = self.render(arg, node, depth, budget)?;
                        budget.append(&mut out, &rendered)?;
                    }
                    budget.append(&mut out, ")")?;
                }
                Expr::Ok { value } | Expr::Group { value } | Expr::Try { value } => {
                    let prefix = match expr {
                        Expr::Ok { .. } => "Ok::<_, Error>(",
                        Expr::Group { .. } => "(",
                        _ => "",
                    };
                    let suffix = match expr {
                        Expr::Try { .. } => "?",
                        _ => ")",
                    };
                    budget.append(&mut out, prefix)?;
                    let rendered = self.render(value, node, depth, budget)?;
                    budget.append(&mut out, &rendered)?;
                    budget.append(&mut out, suffix)?;
                }
                Expr::Lower { field } => {
                    let child = node
                        .get(field)
                        .ok_or_else(|| invalid(format!("missing instruction field {field:?}")))?;
                    let rendered = self.dispatch(child, depth + 1, budget)?;
                    budget.append(&mut out, &rendered)?;
                }
                Expr::String { field } => {
                    let value = node
                        .get(field)
                        .and_then(Value::as_str)
                        .ok_or_else(|| invalid(format!("field {field:?} must be a string")))?;
                    if value.len() > 64 * 1024 {
                        return Err(invalid("string field exceeds 64 KiB"));
                    }
                    write!(
                        BudgetWriter {
                            budget,
                            output: &mut out
                        },
                        "{value:?}"
                    )
                    .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                }
                Expr::Boolean { field } => {
                    let value = node
                        .get(field)
                        .and_then(Value::as_bool)
                        .ok_or_else(|| invalid(format!("field {field:?} must be a boolean")))?;
                    budget.append(&mut out, if value { "true" } else { "false" })?;
                }
            }
            Ok(out)
        })
    }
}

struct BudgetWriter<'a> {
    budget: &'a mut Budget,
    output: &'a mut String,
}
impl fmt::Write for BudgetWriter<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.budget
            .append(self.output, text)
            .map_err(|_| fmt::Error)
    }
}

struct Budget {
    steps: usize,
    bytes: usize,
    frames: usize,
}
impl Budget {
    fn with_frame<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        if self.frames >= 128 {
            return Err(invalid("lowering exceeds 128 combined frames"));
        }
        self.frames += 1;
        let result = f(self);
        self.frames -= 1;
        result
    }
    fn step(&mut self) -> Result<(), Error> {
        self.steps += 1;
        if self.steps > 4096 {
            return Err(invalid("lowering exceeds 4096 steps"));
        }
        Ok(())
    }
    fn append(&mut self, output: &mut String, text: &str) -> Result<(), Error> {
        self.bytes = self.bytes.saturating_add(text.len());
        if self.bytes > 1024 * 1024 {
            return Err(invalid("lowering exceeds 1 MiB rendered bytes"));
        }
        output.push_str(text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, BudgetWriter, Policy};
    use serde_json::json;
    use std::fmt::Write as _;

    #[test]
    fn rejects_duplicates_and_unused_invalid_templates() {
        for text in [
            r#"{"formatVersion":1,"formatVersion":1,"cases":{"x":{"op":"path","value":"x"}}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x"},"x":{"op":"path","value":"x"}}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x","value":"y"}}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x"},"unused":{"op":"call","path":"f","args":[{"op":"path","value":"x()"}]}}}"#,
        ] {
            assert!(Policy::parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn counts_overwritten_duplicate_values_before_typed_decode() {
        let many = vec![r#"{"op":"path","value":"x"}"#; 171].join(",");
        let text = format!(
            r#"{{"formatVersion":1,"cases":{{"x":{{"op":"call","path":"f","args":[{many}]}},"x":{{"op":"path","value":"x"}}}}}}"#
        );
        let error = match Policy::parse(&text) {
            Ok(_) => panic!("accepted excessive duplicate"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("512 JSON nodes"), "{error}");
    }

    #[test]
    fn quoting_obeys_remaining_bytes_without_temporary() {
        let mut budget = Budget {
            steps: 0,
            bytes: 1024 * 1024 - 3,
            frames: 0,
        };
        let mut output = String::new();
        assert!(
            write!(
                BudgetWriter {
                    budget: &mut budget,
                    output: &mut output
                },
                "{:?}",
                "abcdef"
            )
            .is_err()
        );
        assert!(output.len() <= 3);
        assert!(budget.bytes >= 1024 * 1024);
    }

    #[test]
    fn combined_frames_bound_stacked_templates() -> Result<(), Box<dyn std::error::Error>> {
        let mut expr = json!({"op":"lower","field":"child"});
        for _ in 0..8 {
            expr = json!({"op":"group","value":expr});
        }
        let text =
            json!({"formatVersion":1,"cases":{"next":expr,"leaf":{"op":"path","value":"x"}}})
                .to_string();
        let policy = Policy::parse(&text)?;
        let mut node = json!({"op":"leaf"});
        for _ in 0..20 {
            node = json!({"op":"next","child":node});
        }
        let error = match policy.lower(&node) {
            Ok(_) => panic!("accepted excess frames"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("128 combined frames"), "{error}");
        assert_eq!(policy.lower(&json!({"op":"leaf"}))?, "x");
        Ok(())
    }

    #[test]
    fn renders_and_quotes() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"root":{"op":"call","path":"f","args":[{"op":"string","field":"s"},{"op":"boolean","field":"b"},{"op":"try","value":{"op":"group","value":{"op":"lower","field":"child"}}}]},"leaf":{"op":"ok","value":{"op":"path","value":"A::B"}}}}"#,
        )?;
        assert_eq!(
            policy.lower(&json!({"op":"root","s":"a\"\\\n雪","b":true,"child":{"op":"leaf"}}))?,
            "f(\"a\\\"\\\\\\n雪\", true, (Ok::<_, Error>(A::B))?)"
        );
        Ok(())
    }

    #[test]
    fn shared_work_and_depth_limits() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"branch":{"op":"call","path":"f","args":[{"op":"lower","field":"child"},{"op":"lower","field":"child"}]},"leaf":{"op":"path","value":"x"}}}"#,
        )?;
        let mut node = json!({"op":"leaf"});
        for _ in 0..12 {
            node = json!({"op":"branch","child":node});
        }
        assert!(policy.lower(&node).is_err());
        let single = Policy::parse(
            r#"{"formatVersion":1,"cases":{"branch":{"op":"lower","field":"child"},"leaf":{"op":"path","value":"x"}}}"#,
        )?;
        let mut node = json!({"op":"leaf"});
        for _ in 0..63 {
            node = json!({"op":"branch","child":node});
        }
        assert_eq!(single.lower(&node)?, "x");
        node = json!({"op":"branch","child":node});
        assert!(single.lower(&node).is_err());
        Ok(())
    }
}
