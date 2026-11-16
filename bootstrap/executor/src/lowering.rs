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

pub(crate) fn invalid(reason: impl Into<String>) -> Error {
    Error::Invalid {
        reason: reason.into(),
    }
}

pub struct Policy {
    cases: BTreeMap<String, Expr>,
    module: Option<Vec<crate::module::Statement>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPolicy {
    format_version: u32,
    #[serde(deserialize_with = "unique_cases")]
    cases: BTreeMap<String, Expr>,
    #[serde(default, deserialize_with = "module_body")]
    module: Option<Vec<crate::module::Statement>>,
}

fn module_body<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<Vec<crate::module::Statement>>, D::Error> {
    Vec::deserialize(d).map(Some)
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
    Path {
        value: String,
    },
    Call {
        path: String,
        args: Vec<Expr>,
    },
    Ok {
        value: Box<Expr>,
    },
    Group {
        value: Box<Expr>,
    },
    Try {
        value: Box<Expr>,
    },
    Lower {
        field: String,
    },
    String {
        field: String,
    },
    Boolean {
        field: String,
    },
    Text {
        value: String,
    },
    Concat {
        parts: Vec<Expr>,
    },
    Unsigned {
        field: String,
    },
    List {
        field: String,
    },
    Select {
        field: String,
        #[serde(deserialize_with = "unique_cases")]
        cases: BTreeMap<String, Expr>,
    },
    IfEmpty {
        field: String,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
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
            Self::Text { value } if value.len() > 64 * 1024 => Err(invalid("text exceeds 64 KiB")),
            Self::Concat { parts } => {
                for part in parts {
                    part.validate()?;
                }
                Ok(())
            }
            Self::Select { field, cases } => {
                if field.is_empty() || field.len() > 128 {
                    return Err(invalid("selector must contain 1..=128 bytes"));
                }
                validate_cases(cases)
            }
            Self::IfEmpty {
                field,
                then,
                otherwise,
            } => {
                if field.is_empty() || field.len() > 128 {
                    return Err(invalid("selector must contain 1..=128 bytes"));
                }
                then.validate()?;
                otherwise.validate()
            }
            Self::Lower { field }
            | Self::String { field }
            | Self::Boolean { field }
            | Self::Unsigned { field }
            | Self::List { field }
                if field.is_empty() || field.len() > 128 =>
            {
                Err(invalid("selector must contain 1..=128 bytes"))
            }
            _ => Ok(()),
        }
    }
}

fn validate_cases(cases: &BTreeMap<String, Expr>) -> Result<(), Error> {
    if cases.is_empty() || cases.len() > 64 {
        return Err(invalid("policy must contain 1..=64 cases"));
    }
    for (key, expr) in cases {
        if key.is_empty() || key.len() > 64 {
            return Err(invalid("case key must contain 1..=64 bytes"));
        }
        expr.validate()?;
    }
    Ok(())
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
        validate_cases(&policy.cases)?;
        if let Some(body) = &policy.module {
            crate::module::validate(body)?;
        }
        Ok(Self {
            cases: policy.cases,
            module: policy.module,
        })
    }

    /// Lower one instruction with fresh, shared budgets.
    pub fn lower(&self, instruction: &Value) -> Result<String, Error> {
        Context::run(instruction, self)
    }
}

pub trait Driver {
    fn assemble(&self, _frame: &mut crate::module::Frame<'_>) -> Result<(), Error> {
        Err(invalid("missing module program"))
    }
    fn render(&self, ctx: &mut Context, node: &Value, depth: usize) -> Result<String, Error>;
}

pub fn instruction_op(node: &Value) -> Result<&str, Error> {
    node.get("op")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("instruction requires string op"))
}

pub fn missing_case(op: &str) -> Error {
    invalid(format!("missing lowering case for {op:?}"))
}

pub fn missing_selection(tag: &str) -> Error {
    invalid(format!("missing lowering selection for {tag:?}"))
}

impl Driver for Policy {
    fn assemble(&self, frame: &mut crate::module::Frame<'_>) -> Result<(), Error> {
        let body = self
            .module
            .as_deref()
            .ok_or_else(|| invalid("missing module program"))?;
        crate::module::execute(body, frame, self)
    }
    fn render(&self, ctx: &mut Context, node: &Value, depth: usize) -> Result<String, Error> {
        let op = instruction_op(node)?;
        self.cases
            .get(op)
            .ok_or_else(|| missing_case(op))?
            .render(ctx, node, depth, self)
    }
}

impl Expr {
    fn render(
        &self,
        ctx: &mut Context,
        node: &Value,
        depth: usize,
        driver: &dyn Driver,
    ) -> Result<String, Error> {
        match self {
            Self::Path { value } => ctx.path(value),
            Self::Call { path, args } => ctx.call(path, |call| {
                for arg in args {
                    call.arg(|ctx| arg.render(ctx, node, depth, driver))?;
                }
                Ok(())
            }),
            Self::Ok { value } => ctx.ok(|ctx| value.render(ctx, node, depth, driver)),
            Self::Group { value } => ctx.group(|ctx| value.render(ctx, node, depth, driver)),
            Self::Try { value } => ctx.try_value(|ctx| value.render(ctx, node, depth, driver)),
            Self::Lower { field } => ctx.child(node, field, depth, driver),
            Self::String { field } => ctx.string(node, field),
            Self::Boolean { field } => ctx.boolean(node, field),
            Self::Text { value } => ctx.text(value),
            Self::Concat { parts } => ctx.concat(|call| {
                for part in parts {
                    call.arg(|ctx| part.render(ctx, node, depth, driver))?;
                }
                Ok(())
            }),
            Self::Unsigned { field } => ctx.unsigned(node, field),
            Self::List { field } => ctx.list(node, field, depth, driver),
            Self::Select { field, cases } => ctx.select(node, field, |ctx, tag| {
                cases
                    .get(tag)
                    .ok_or_else(|| missing_selection(tag))?
                    .render(ctx, node, depth, driver)
            }),
            Self::IfEmpty {
                field,
                then,
                otherwise,
            } => ctx.if_empty(
                node,
                field,
                |ctx| then.render(ctx, node, depth, driver),
                |ctx| otherwise.render(ctx, node, depth, driver),
            ),
        }
    }
}

/// Shared resource and rendering kernel for lowering drivers.
pub struct Context {
    budget: Budget,
}
impl Context {
    pub fn run(node: &Value, driver: &dyn Driver) -> Result<String, Error> {
        let mut ctx = Self {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 0,
            },
        };
        ctx.dispatch(node, 1, driver)
    }
    fn scope<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        if self.budget.frames >= 128 {
            return Err(invalid("lowering exceeds 128 combined frames"));
        }
        self.budget.frames += 1;
        let result = f(self);
        self.budget.frames -= 1;
        result
    }
    fn expression<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        self.scope(|ctx| {
            ctx.budget.step()?;
            f(ctx)
        })
    }
    fn dispatch(
        &mut self,
        node: &Value,
        depth: usize,
        driver: &dyn Driver,
    ) -> Result<String, Error> {
        self.scope(|ctx| {
            ctx.budget.step()?;
            if depth > 64 {
                return Err(invalid("instruction depth exceeds 64"));
            }
            driver.render(ctx, node, depth)
        })
    }
    pub fn path(&mut self, path: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let mut out = String::new();
            ctx.budget.append(&mut out, path)?;
            Ok(out)
        })
    }
    pub fn text(&mut self, value: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let mut out = String::new();
            ctx.budget.append(&mut out, value)?;
            Ok(out)
        })
    }
    pub fn unsigned(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_u64)
                .ok_or_else(|| invalid(format!("field {field:?} must be an unsigned integer")))?;
            let mut out = String::new();
            write!(
                BudgetWriter {
                    budget: &mut ctx.budget,
                    output: &mut out
                },
                "{value}"
            )
            .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
            Ok(out)
        })
    }
    pub fn list(
        &mut self,
        node: &Value,
        field: &str,
        depth: usize,
        driver: &dyn Driver,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let values = array_field(node, field)?;
            let mut out = String::new();
            for value in values {
                let rendered = ctx.dispatch(value, depth + 1, driver)?;
                ctx.budget.append(&mut out, &rendered)?;
            }
            Ok(out)
        })
    }
    pub fn if_empty(
        &mut self,
        node: &Value,
        field: &str,
        then: impl FnOnce(&mut Context) -> Result<String, Error>,
        otherwise: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let empty = array_field(node, field)?.is_empty();
            let rendered = if empty { then(ctx)? } else { otherwise(ctx)? };
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn select(
        &mut self,
        node: &Value,
        field: &str,
        branch: impl FnOnce(&mut Context, &str) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let tag = node
                .get(field)
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(format!("field {field:?} must be a string")))?;
            let rendered = branch(ctx, tag)?;
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn string(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(format!("field {field:?} must be a string")))?;
            if value.len() > 64 * 1024 {
                return Err(invalid("string field exceeds 64 KiB"));
            }
            let mut out = String::new();
            write!(
                BudgetWriter {
                    budget: &mut ctx.budget,
                    output: &mut out
                },
                "{value:?}"
            )
            .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
            Ok(out)
        })
    }
    pub fn boolean(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_bool)
                .ok_or_else(|| invalid(format!("field {field:?} must be a boolean")))?;
            let mut out = String::new();
            ctx.budget
                .append(&mut out, if value { "true" } else { "false" })?;
            Ok(out)
        })
    }
    pub fn child(
        &mut self,
        node: &Value,
        field: &str,
        depth: usize,
        driver: &dyn Driver,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let child = node
                .get(field)
                .ok_or_else(|| invalid(format!("missing instruction field {field:?}")))?;
            let rendered = ctx.dispatch(child, depth + 1, driver)?;
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    fn wrap(
        &mut self,
        prefix: &str,
        suffix: &str,
        f: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let mut out = String::new();
            ctx.budget.append(&mut out, prefix)?;
            let rendered = f(ctx)?;
            ctx.budget.append(&mut out, &rendered)?;
            ctx.budget.append(&mut out, suffix)?;
            Ok(out)
        })
    }
    pub fn ok(
        &mut self,
        f: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.wrap("Ok::<_, Error>(", ")", f)
    }
    pub fn group(
        &mut self,
        f: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.wrap("(", ")", f)
    }
    pub fn try_value(
        &mut self,
        f: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.wrap("", "?", f)
    }
    pub fn call(
        &mut self,
        path: &str,
        f: impl FnOnce(&mut Call<'_>) -> Result<(), Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let mut out = String::new();
            ctx.budget.append(&mut out, path)?;
            ctx.budget.append(&mut out, "(")?;
            f(&mut Call {
                ctx,
                out: &mut out,
                first: true,
                separator: ", ",
            })?;
            ctx.budget.append(&mut out, ")")?;
            Ok(out)
        })
    }
    pub fn concat(
        &mut self,
        f: impl FnOnce(&mut Call<'_>) -> Result<(), Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let mut out = String::new();
            f(&mut Call {
                ctx,
                out: &mut out,
                first: true,
                separator: "",
            })?;
            Ok(out)
        })
    }
}

fn array_field<'a>(node: &'a Value, field: &str) -> Result<&'a Vec<Value>, Error> {
    node.get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("field {field:?} must be an array")))
}

pub struct Call<'a> {
    ctx: &'a mut Context,
    out: &'a mut String,
    first: bool,
    separator: &'static str,
}
impl Call<'_> {
    pub fn arg(
        &mut self,
        f: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<(), Error> {
        if !self.first {
            self.ctx.budget.append(self.out, self.separator)?;
        }
        self.first = false;
        let rendered = f(self.ctx)?;
        self.ctx.budget.append(self.out, &rendered)
    }
}

/// Compile a fully validated policy into a standalone lowering driver module.
pub fn generate(policy_json: &str) -> Result<String, Error> {
    let policy = Policy::parse(policy_json)?;
    let mut out = String::from(
        "// Generated lowering driver. Do not edit.\nuse bootstrap_executor::lowering::{self, Context, Driver, Error};\nuse serde_json::Value;\n\npub struct Compiled;\n\nimpl Driver for Compiled {\n    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {\n        match lowering::instruction_op(_node)? {\n",
    );
    for (op, expr) in &policy.cases {
        write!(out, "            {op:?} => ").expect("write to string");
        expr.emit(&mut out);
        out.push_str(",\n");
    }
    out.push_str("            op => Err(lowering::missing_case(op)),\n        }\n    }\n");
    if let Some(body) = &policy.module {
        let frame = if body.is_empty() { "_frame" } else { "frame" };
        writeln!(out, "    fn assemble(&self, {frame}: &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {{").expect("write to string");
        crate::module::emit(body, &mut out);
        out.push_str("Ok(())\n}\n");
    }
    out.push_str("}\n\npub fn lower(node: &Value) -> Result<String, Error> {\n    Context::run(node, &Compiled)\n}\n");
    Ok(out)
}
impl Expr {
    fn emit(&self, out: &mut String) {
        match self {
            Self::Path { value } => write!(out, "ctx.path({value:?})").expect("write to string"),
            Self::Text { value } => write!(out, "ctx.text({value:?})").expect("write to string"),
            Self::Unsigned { field } => {
                write!(out, "ctx.unsigned(_node, {field:?})").expect("write to string")
            }
            Self::List { field } => {
                write!(out, "ctx.list(_node, {field:?}, _depth, self)").expect("write to string")
            }
            Self::Select { field, cases } => {
                write!(out, "ctx.select(_node, {field:?}, |ctx, tag| match tag {{ ")
                    .expect("write to string");
                for (tag, expr) in cases {
                    write!(out, "{tag:?} => ").expect("write to string");
                    expr.emit(out);
                    out.push_str(", ");
                }
                out.push_str("tag => Err(lowering::missing_selection(tag)), })");
            }
            Self::IfEmpty {
                field,
                then,
                otherwise,
            } => {
                write!(out, "ctx.if_empty(_node, {field:?}, |ctx| ").expect("write to string");
                then.emit(out);
                out.push_str(", |ctx| ");
                otherwise.emit(out);
                out.push(')');
            }
            Self::Concat { parts } => {
                let name = if parts.is_empty() { "_parts" } else { "parts" };
                writeln!(out, "ctx.concat(|{name}| {{").expect("write to string");
                for part in parts {
                    out.push_str("parts.arg(|ctx| ");
                    part.emit(out);
                    out.push_str(")?;\n");
                }
                out.push_str("Ok(())\n})");
            }
            Self::String { field } => {
                write!(out, "ctx.string(_node, {field:?})").expect("write to string")
            }
            Self::Boolean { field } => {
                write!(out, "ctx.boolean(_node, {field:?})").expect("write to string")
            }
            Self::Lower { field } => {
                write!(out, "ctx.child(_node, {field:?}, _depth, self)").expect("write to string")
            }
            Self::Ok { value } | Self::Group { value } | Self::Try { value } => {
                let method = match self {
                    Self::Ok { .. } => "ok",
                    Self::Group { .. } => "group",
                    _ => "try_value",
                };
                write!(out, "ctx.{method}(|ctx| ").expect("write to string");
                value.emit(out);
                out.push(')');
            }
            Self::Call { path, args } => {
                let name = if args.is_empty() { "_call" } else { "call" };
                write!(out, "ctx.call({path:?}, |{name}| {{ ").expect("write to string");
                for arg in args {
                    out.push_str("call.arg(|ctx| ");
                    arg.emit(out);
                    out.push_str(")?; ");
                }
                out.push_str("Ok(()) })");
            }
        }
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
    use super::{Budget, BudgetWriter, Context, Policy};
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
    fn collection_primitives_keep_exact_budget_precedence() {
        for (remaining, expected) in [
            (2, "lowering: field \"n\" must be an unsigned integer"),
            (1, "lowering: lowering exceeds 1 MiB rendered bytes"),
        ] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            let result = ctx.concat(|parts| {
                parts.arg(|ctx| ctx.text("x"))?;
                parts.arg(|ctx| ctx.unsigned(&json!({"n":false}), "n"))
            });
            assert_eq!(result.expect_err("concat ordering").to_string(), expected);
            assert_eq!(ctx.budget.frames, 0);
        }
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 1024 * 1024,
                frames: 0,
            },
        };
        assert_eq!(
            ctx.if_empty(
                &json!({"a":false}),
                "a",
                |ctx| ctx.text("x"),
                |ctx| ctx.text("y")
            )
            .expect_err("array check first")
            .to_string(),
            "lowering: field \"a\" must be an array"
        );
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 127,
            },
        };
        assert!(
            ctx.concat(|parts| parts.arg(|ctx| ctx.text("x")))
                .expect_err("combined collection frames")
                .to_string()
                .contains("128 combined frames")
        );
        assert_eq!(ctx.budget.frames, 127);
    }

    #[test]
    fn fixed_frame_and_byte_boundaries_preserve_error_precedence() {
        let policy = Policy::parse(r#"{"formatVersion":1,"cases":{"leaf":{"op":"path","value":"x"},"call":{"op":"call","path":"f","args":[{"op":"path","value":"x"}]},"bad":{"op":"call","path":"f","args":[{"op":"path","value":"x"},{"op":"string","field":"s"}]}}}"#).expect("boundary policy");
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 126,
            },
        };
        assert_eq!(
            ctx.dispatch(&json!({"op":"leaf"}), 1, &policy)
                .expect("128 frames accepted"),
            "x"
        );
        assert_eq!(ctx.budget.frames, 126);
        ctx.budget.frames = 127;
        assert_eq!(
            ctx.dispatch(&json!({"op":"leaf"}), 1, &policy)
                .expect_err("129th frame")
                .to_string(),
            "lowering: lowering exceeds 128 combined frames"
        );
        assert_eq!(ctx.budget.frames, 127);
        for (remaining, expected) in [
            (5, Ok("f(x)")),
            (4, Err("lowering: lowering exceeds 1 MiB rendered bytes")),
        ] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            assert_eq!(
                ctx.dispatch(&json!({"op":"call"}), 1, &policy)
                    .map_err(|e| e.to_string()),
                expected.map(str::to_owned).map_err(str::to_owned)
            );
            assert_eq!(ctx.budget.frames, 0);
        }
        // f( + x expression + x append + comma-space = six charged bytes.
        // At exactly the cap the malformed second argument wins; one byte less
        // rejects its delimiter before the argument is inspected.
        for (remaining, expected) in [
            (6, "lowering: field \"s\" must be a string"),
            (5, "lowering: lowering exceeds 1 MiB rendered bytes"),
        ] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            assert_eq!(
                ctx.dispatch(&json!({"op":"bad","s":false}), 1, &policy)
                    .expect_err("boundary failure")
                    .to_string(),
                expected
            );
            assert_eq!(ctx.budget.frames, 0);
        }
    }

    #[test]
    fn extended_expressions_validate_and_share_budgets() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(&json!({"formatVersion":1,"cases":{
            "root":{"op":"concat","parts":[{"op":"text","value":"["},{"op":"list","field":"items"},{"op":"ifEmpty","field":"items","then":{"op":"text","value":"empty"},"else":{"op":"text","value":"]"}}]},
            "item":{"op":"unsigned","field":"n"}
        }}).to_string())?;
        assert_eq!(policy.lower(&json!({"op":"root","items":[]}))?, "[empty");
        assert_eq!(
            policy.lower(
                &json!({"op":"root","items":[{"op":"item","n":0},{"op":"item","n":u64::MAX}]})
            )?,
            "[018446744073709551615]"
        );
        assert!(
            policy
                .lower(&json!({"op":"root","items":null}))
                .expect_err("array required")
                .to_string()
                .contains("must be an array")
        );
        for value in [json!(-1), json!(1.5), json!("1")] {
            assert!(
                policy
                    .lower(&json!({"op":"item","n":value}))
                    .expect_err("unsigned integer required")
                    .to_string()
                    .contains("unsigned integer")
            );
        }
        let items = vec![json!({"op":"item","n":1}); 2050];
        assert!(
            policy
                .lower(&json!({"op":"root","items":items}))
                .expect_err("shared fuel exhausted")
                .to_string()
                .contains("4096 steps")
        );
        let invalid = json!({"formatVersion":1,"cases":{"x":{"op":"ifEmpty","field":"a","then":{"op":"text","value":""},"else":{"op":"lower","field":""}}}}).to_string();
        assert!(Policy::parse(&invalid).is_err());
        let generated = super::generate(
            &json!({"formatVersion":1,"cases":{"x":{"op":"concat","parts":[]}}}).to_string(),
        )?;
        assert!(generated.contains("ctx.concat(|_parts|"));
        Ok(())
    }

    #[test]
    fn selection_validates_and_renders() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"root":{"op":"select","field":"tag","cases":{"a":{"op":"lower","field":"child"},"b":{"op":"text","value":"B"}}},"leaf":{"op":"text","value":"ok"}}}"#,
        )?;
        assert_eq!(
            policy.lower(&json!({"op":"root","tag":"a","child":{"op":"leaf"}}))?,
            "ok"
        );
        assert_eq!(policy.lower(&json!({"op":"root","tag":"b"}))?, "B");
        assert_eq!(
            policy
                .lower(&json!({"op":"root","tag":false}))
                .expect_err("string")
                .to_string(),
            "lowering: field \"tag\" must be a string"
        );
        assert_eq!(
            policy
                .lower(&json!({"op":"root","tag":"missing"}))
                .expect_err("missing")
                .to_string(),
            "lowering: missing lowering selection for \"missing\""
        );
        for cases in [
            r#"{}"#,
            r#"{"":{"op":"text","value":"x"}}"#,
            r#"{"a":{"op":"text","value":"x"},"a":{"op":"text","value":"y"}}"#,
            r#"{"a":{"op":"text","value":"x"},"b":{"op":"path","value":"bad()"}}"#,
        ] {
            let text = format!(
                r#"{{"formatVersion":1,"cases":{{"x":{{"op":"select","field":"tag","cases":{cases}}}}}}}"#
            );
            assert!(Policy::parse(&text).is_err(), "{text}");
        }
        for size in [0, 129] {
            let text = json!({"formatVersion":1,"cases":{"x":{"op":"select","field":"x".repeat(size),"cases":{"a":{"op":"text","value":"x"}}}}}).to_string();
            assert!(Policy::parse(&text).is_err());
        }
        let generated = super::generate(
            r#"{"formatVersion":1,"cases":{"x":{"op":"select","field":"a\"b","cases":{"x\"y":{"op":"text","value":"z"}}}}}"#,
        )?;
        assert!(generated.contains(r#"ctx.select(_node, "a\"b", |ctx, tag| match tag"#));
        assert!(generated.contains(r#""x\"y" => ctx.text("z")"#));
        Ok(())
    }

    #[test]
    fn selection_charges_copy_and_unwinds() {
        for (remaining, succeeds) in [(2, true), (1, false)] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            let result = ctx.select(&json!({"tag":"a"}), "tag", |ctx, _| ctx.text("x"));
            assert_eq!(result.is_ok(), succeeds);
            assert_eq!(ctx.budget.steps, 2);
            assert_eq!(ctx.budget.frames, 0);
        }
        let mut ctx = Context {
            budget: Budget {
                steps: 4095,
                bytes: 0,
                frames: 0,
            },
        };
        assert!(
            ctx.select(&json!({"tag":"a"}), "tag", |ctx, _| ctx.text("x"))
                .expect_err("fuel")
                .to_string()
                .contains("4096 steps")
        );
        assert_eq!(ctx.budget.frames, 0);
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
