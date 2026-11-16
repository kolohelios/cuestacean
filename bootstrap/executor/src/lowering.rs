//! Typed, CUE-authored instruction lowering policy.
use std::{collections::BTreeMap, fmt, fmt::Write as _};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
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
    aliases: BTreeMap<String, String>,
    module: Option<Vec<crate::module::Statement>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPolicy {
    format_version: u32,
    #[serde(deserialize_with = "unique_cases")]
    cases: BTreeMap<String, Expr>,
    #[serde(default, deserialize_with = "unique_cases")]
    aliases: BTreeMap<String, String>,
    #[serde(default, deserialize_with = "module_body")]
    module: Option<Vec<crate::module::Statement>>,
}

fn module_body<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<Vec<crate::module::Statement>>, D::Error> {
    Vec::deserialize(d).map(Some)
}

// Unlike a struct, a map normally accepts duplicate keys during deserialization.
fn unique_cases<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, T>, D::Error> {
    struct Cases<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Cases<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map of unique lowering cases")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut cases = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if cases.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!("duplicate case {key:?}")));
                }
            }
            Ok(cases)
        }
    }
    deserializer.deserialize_map(Cases(std::marker::PhantomData))
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Expr {
    Path {
        value: String,
    },
    Call {
        path: String,
        #[serde(serialize_with = "serialize_call_args")]
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
        #[serde(serialize_with = "serialize_concat_parts")]
        parts: Vec<Expr>,
    },
    Unsigned {
        field: String,
    },
    List {
        field: String,
    },
    Signed {
        field: String,
    },
    Join {
        field: String,
        separator: String,
    },
    IfZero {
        field: String,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
    IfTrue {
        field: String,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
    Bind {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Get {
        name: String,
    },
    IfEqual {
        left: Box<Expr>,
        right: Box<Expr>,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
    BytePattern {
        field: String,
    },
    CharPattern {
        field: String,
    },
    Select {
        field: String,
        #[serde(
            deserialize_with = "unique_cases",
            serialize_with = "serialize_select_cases"
        )]
        cases: BTreeMap<String, Expr>,
    },
    Identifier {
        field: String,
        #[serde(default)]
        snake: bool,
    },
    IfEmpty {
        field: String,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
}

#[derive(Serialize)]
struct Role<'a> {
    op: &'static str,
    value: &'a Expr,
}

fn serialize_roles<S: Serializer>(
    values: &[Expr],
    op: &'static str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    values
        .iter()
        .map(|value| Role { op, value })
        .collect::<Vec<_>>()
        .serialize(serializer)
}
fn serialize_call_args<S: Serializer>(values: &[Expr], serializer: S) -> Result<S::Ok, S::Error> {
    serialize_roles(values, "callArgument", serializer)
}
fn serialize_concat_parts<S: Serializer>(
    values: &[Expr],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serialize_roles(values, "concatArgument", serializer)
}
#[derive(Serialize)]
struct SelectRole<'a> {
    op: &'static str,
    tag: &'a str,
    value: &'a Expr,
}
fn serialize_select_cases<S: Serializer>(
    cases: &BTreeMap<String, Expr>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    cases
        .iter()
        .map(|(tag, value)| SelectRole {
            op: "selectCase",
            tag,
            value,
        })
        .collect::<Vec<_>>()
        .serialize(serializer)
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
    fn validate_scoped<'a>(&'a self, env: &[&'a str]) -> Result<(), Error> {
        match self {
            Self::Bind { name, value, body } => {
                let mut bytes = name.bytes();
                if name.len() > 64
                    || !matches!(bytes.next(), Some(b'a'..=b'z' | b'_'))
                    || !bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                {
                    return Err(invalid("invalid binding name"));
                }
                value.validate_scoped(env)?;
                let mut inner = env.to_vec();
                inner.push(name);
                body.validate_scoped(&inner)
            }
            Self::Get { name } => {
                if env.contains(&name.as_str()) {
                    Ok(())
                } else {
                    Err(invalid("unknown binding"))
                }
            }
            Self::IfEqual {
                left,
                right,
                then,
                otherwise,
            } => {
                left.validate_scoped(env)?;
                right.validate_scoped(env)?;
                then.validate_scoped(env)?;
                otherwise.validate_scoped(env)
            }
            Self::Path { value } if !valid_path(value) => Err(invalid("invalid Rust path")),
            Self::Call { path, args } => {
                if !valid_path(path) {
                    return Err(invalid("invalid Rust path"));
                }
                for arg in args {
                    arg.validate_scoped(env)?;
                }
                Ok(())
            }
            Self::Ok { value } | Self::Group { value } | Self::Try { value } => {
                value.validate_scoped(env)
            }
            Self::Text { value } if value.len() > 64 * 1024 => Err(invalid("text exceeds 64 KiB")),
            Self::Concat { parts } => {
                for part in parts {
                    part.validate_scoped(env)?;
                }
                Ok(())
            }
            Self::Select { field, cases } => {
                if field.is_empty() || field.len() > 128 {
                    return Err(invalid("selector must contain 1..=128 bytes"));
                }
                validate_cases_scoped(cases, env)
            }
            Self::IfEmpty {
                field,
                then,
                otherwise,
            }
            | Self::IfZero {
                field,
                then,
                otherwise,
            }
            | Self::IfTrue {
                field,
                then,
                otherwise,
            } => {
                if field.is_empty() || field.len() > 128 {
                    return Err(invalid("selector must contain 1..=128 bytes"));
                }
                then.validate_scoped(env)?;
                otherwise.validate_scoped(env)
            }
            Self::Join { field, separator } => {
                if field.is_empty() || field.len() > 128 {
                    return Err(invalid("selector must contain 1..=128 bytes"));
                }
                if separator.len() > 128 {
                    return Err(invalid("separator exceeds 128 bytes"));
                }
                Ok(())
            }
            Self::Lower { field }
            | Self::Signed { field }
            | Self::String { field }
            | Self::Boolean { field }
            | Self::Unsigned { field }
            | Self::List { field }
            | Self::BytePattern { field }
            | Self::CharPattern { field }
            | Self::Identifier { field, .. }
                if field.is_empty() || field.len() > 128 =>
            {
                Err(invalid("selector must contain 1..=128 bytes"))
            }
            _ => Ok(()),
        }
    }
}

fn validate_cases(cases: &BTreeMap<String, Expr>) -> Result<(), Error> {
    validate_cases_scoped(cases, &[])
}

fn validate_cases_scoped<'a>(
    cases: &'a BTreeMap<String, Expr>,
    env: &[&'a str],
) -> Result<(), Error> {
    if cases.is_empty() || cases.len() > 64 {
        return Err(invalid("policy must contain 1..=64 cases"));
    }
    for (key, expr) in cases {
        if key.is_empty() || key.len() > 64 {
            return Err(invalid("case key must contain 1..=64 bytes"));
        }
        expr.validate_scoped(env)?;
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
        if policy.cases.len() + policy.aliases.len() > 64 {
            return Err(invalid("policy must contain 1..=64 cases and aliases"));
        }
        for (alias, target) in &policy.aliases {
            if alias.is_empty() || alias.len() > 64 {
                return Err(invalid("alias key must contain 1..=64 bytes"));
            }
            if policy.cases.contains_key(alias) || !policy.cases.contains_key(target) {
                return Err(invalid(
                    "alias must not collide and must target a canonical case",
                ));
            }
        }
        if let Some(body) = &policy.module {
            crate::module::validate(body)?;
        }
        Ok(Self {
            cases: policy.cases,
            aliases: policy.aliases,
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
        let canonical = self.aliases.get(op).map_or(op, String::as_str);
        self.cases
            .get(canonical)
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
        self.render_scoped(ctx, node, depth, driver, &[])
    }

    fn render_scoped<'a>(
        &'a self,
        ctx: &mut Context,
        node: &Value,
        depth: usize,
        driver: &dyn Driver,
        env: &[(&'a str, &str)],
    ) -> Result<String, Error> {
        match self {
            Self::Bind { name, value, body } => ctx.bind(
                |ctx| value.render_scoped(ctx, node, depth, driver, env),
                |ctx, rendered| {
                    let mut inner = env.to_vec();
                    inner.push((name, rendered));
                    body.render_scoped(ctx, node, depth, driver, &inner)
                },
            ),
            Self::Get { name } => ctx.rendered(
                env.iter()
                    .rev()
                    .find(|(key, _)| *key == name)
                    .ok_or_else(|| invalid("unknown binding"))?
                    .1,
            ),
            Self::IfEqual {
                left,
                right,
                then,
                otherwise,
            } => ctx.if_equal(
                |ctx| left.render_scoped(ctx, node, depth, driver, env),
                |ctx| right.render_scoped(ctx, node, depth, driver, env),
                |ctx| then.render_scoped(ctx, node, depth, driver, env),
                |ctx| otherwise.render_scoped(ctx, node, depth, driver, env),
            ),
            Self::IfTrue {
                field,
                then,
                otherwise,
            } => ctx.if_true(
                node,
                field,
                |ctx| then.render_scoped(ctx, node, depth, driver, env),
                |ctx| otherwise.render_scoped(ctx, node, depth, driver, env),
            ),
            Self::Identifier { field, snake } => ctx.identifier(node, field, *snake),
            Self::BytePattern { field } => ctx.byte_pattern(node, field),
            Self::CharPattern { field } => ctx.char_pattern(node, field),
            Self::Path { value } => ctx.path(value),
            Self::Call { path, args } => ctx.call(path, |call| {
                for arg in args {
                    call.arg(|ctx| arg.render_scoped(ctx, node, depth, driver, env))?;
                }
                Ok(())
            }),
            Self::Ok { value } => ctx.ok(|ctx| value.render_scoped(ctx, node, depth, driver, env)),
            Self::Group { value } => {
                ctx.group(|ctx| value.render_scoped(ctx, node, depth, driver, env))
            }
            Self::Try { value } => {
                ctx.try_value(|ctx| value.render_scoped(ctx, node, depth, driver, env))
            }
            Self::Lower { field } => ctx.child(node, field, depth, driver),
            Self::String { field } => ctx.string(node, field),
            Self::Boolean { field } => ctx.boolean(node, field),
            Self::Text { value } => ctx.text(value),
            Self::Concat { parts } => ctx.concat(|call| {
                for part in parts {
                    call.arg(|ctx| part.render_scoped(ctx, node, depth, driver, env))?;
                }
                Ok(())
            }),
            Self::Unsigned { field } => ctx.unsigned(node, field),
            Self::Signed { field } => ctx.signed(node, field),
            Self::List { field } => ctx.list(node, field, depth, driver),
            Self::Join { field, separator } => ctx.join(node, field, separator, depth, driver),
            Self::IfZero {
                field,
                then,
                otherwise,
            } => ctx.if_zero(
                node,
                field,
                |ctx| then.render_scoped(ctx, node, depth, driver, env),
                |ctx| otherwise.render_scoped(ctx, node, depth, driver, env),
            ),
            Self::Select { field, cases } => ctx.select(node, field, |ctx, tag| {
                cases
                    .get(tag)
                    .ok_or_else(|| missing_selection(tag))?
                    .render_scoped(ctx, node, depth, driver, env)
            }),
            Self::IfEmpty {
                field,
                then,
                otherwise,
            } => ctx.if_empty(
                node,
                field,
                |ctx| then.render_scoped(ctx, node, depth, driver, env),
                |ctx| otherwise.render_scoped(ctx, node, depth, driver, env),
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
    pub fn signed(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_i64)
                .ok_or_else(|| invalid(format!("field {field:?} must be a signed integer")))?;
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
        self.join(node, field, "", depth, driver)
    }
    pub fn join(
        &mut self,
        node: &Value,
        field: &str,
        separator: &str,
        depth: usize,
        driver: &dyn Driver,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let values = array_field(node, field)?;
            let mut out = String::new();
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    ctx.budget.append(&mut out, separator)?;
                }
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
    pub fn if_zero(
        &mut self,
        node: &Value,
        field: &str,
        then: impl FnOnce(&mut Context) -> Result<String, Error>,
        otherwise: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_u64)
                .ok_or_else(|| invalid(format!("field {field:?} must be an unsigned integer")))?;
            let rendered = if value == 0 {
                then(ctx)?
            } else {
                otherwise(ctx)?
            };
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn bind(
        &mut self,
        value: impl FnOnce(&mut Context) -> Result<String, Error>,
        body: impl FnOnce(&mut Context, &str) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = value(ctx)?;
            let rendered = body(ctx, &value)?;
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn rendered(&mut self, value: &str) -> Result<String, Error> {
        self.text(value)
    }
    pub fn if_equal(
        &mut self,
        left: impl FnOnce(&mut Context) -> Result<String, Error>,
        right: impl FnOnce(&mut Context) -> Result<String, Error>,
        then: impl FnOnce(&mut Context) -> Result<String, Error>,
        otherwise: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let left = left(ctx)?;
            let right = right(ctx)?;
            let rendered = if left == right {
                then(ctx)?
            } else {
                otherwise(ctx)?
            };
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn if_true(
        &mut self,
        node: &Value,
        field: &str,
        then: impl FnOnce(&mut Context) -> Result<String, Error>,
        otherwise: impl FnOnce(&mut Context) -> Result<String, Error>,
    ) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node
                .get(field)
                .and_then(Value::as_bool)
                .ok_or_else(|| invalid(format!("field {field:?} must be a boolean")))?;
            let rendered = if value { then(ctx)? } else { otherwise(ctx)? };
            let mut out = String::new();
            ctx.budget.append(&mut out, &rendered)?;
            Ok(out)
        })
    }
    pub fn byte_pattern(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let values = pattern_field(node, field)?;
            let mut bytes = Vec::with_capacity(values.len());
            for value in values {
                let byte = value.as_u64().filter(|n| *n <= 127).ok_or_else(|| {
                    invalid(format!(
                        "field {field:?} must contain ASCII unsigned integers"
                    ))
                })?;
                bytes.push(byte);
            }
            bytes.sort_unstable();
            bytes.dedup();
            let mut out = String::new();
            let mut writer = BudgetWriter {
                budget: &mut ctx.budget,
                output: &mut out,
            };
            let mut i = 0;
            while i < bytes.len() {
                if i > 0 {
                    writer
                        .write_str(" | ")
                        .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                }
                let mut end = i;
                while end + 1 < bytes.len() && bytes[end + 1] == bytes[end] + 1 {
                    end += 1;
                }
                if end - i >= 2 {
                    write!(writer, "{}..={}", bytes[i], bytes[end])
                        .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                    i = end + 1;
                } else {
                    write!(writer, "{}", bytes[i])
                        .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                    i += 1;
                }
            }
            Ok(out)
        })
    }
    pub fn char_pattern(&mut self, node: &Value, field: &str) -> Result<String, Error> {
        self.expression(|ctx| {
            let values = pattern_field(node, field)?;
            let mut chars = Vec::with_capacity(values.len());
            for value in values {
                let mut iter = value
                    .as_str()
                    .ok_or_else(|| {
                        invalid(format!(
                            "field {field:?} must contain single scalar strings"
                        ))
                    })?
                    .chars();
                let c = iter.next().ok_or_else(|| {
                    invalid(format!(
                        "field {field:?} must contain single scalar strings"
                    ))
                })?;
                if iter.next().is_some() {
                    return Err(invalid(format!(
                        "field {field:?} must contain single scalar strings"
                    )));
                }
                chars.push(c);
            }
            chars.sort_unstable();
            chars.dedup();
            let mut out = String::new();
            let mut writer = BudgetWriter {
                budget: &mut ctx.budget,
                output: &mut out,
            };
            for (i, c) in chars.iter().enumerate() {
                if i > 0 {
                    writer
                        .write_str(" | ")
                        .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                }
                write!(writer, "{c:?}")
                    .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
            }
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
    /// Render a checked ASCII identifier component, not a scanner source injection API.
    pub fn identifier(&mut self, node: &Value, field: &str, snake: bool) -> Result<String, Error> {
        self.expression(|ctx| {
            let value = node.get(field).and_then(Value::as_str).ok_or_else(|| {
                invalid(format!("field {field:?} must be an identifier component"))
            })?;
            let mut bytes = value.bytes();
            if value.len() > 64
                || !matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
                || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                return Err(invalid(format!(
                    "field {field:?} must be an identifier component"
                )));
            }
            let mut out = String::new();
            let mut writer = BudgetWriter {
                budget: &mut ctx.budget,
                output: &mut out,
            };
            for (i, b) in value.bytes().enumerate() {
                if snake && i > 0 && b.is_ascii_uppercase() {
                    writer
                        .write_char('_')
                        .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
                }
                let b = if snake { b.to_ascii_lowercase() } else { b };
                writer
                    .write_char(char::from(b))
                    .map_err(|_| invalid("lowering exceeds 1 MiB rendered bytes"))?;
            }
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

fn pattern_field<'a>(node: &'a Value, field: &str) -> Result<&'a [Value], Error> {
    let values = array_field(node, field)?;
    if values.is_empty() || values.len() > 128 {
        return Err(invalid(format!(
            "field {field:?} must contain 1..=128 values"
        )));
    }
    Ok(values)
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
    #[cfg(feature = "compiled-lowering")]
    {
        generate_driver(&policy, &crate::compiled_meta_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        let compiler = Policy::parse(include_str!(concat!(
            env!("CUESTACEAN_BOOTSTRAP_JSON"),
            "/meta-lowering.json"
        )))?;
        generate_driver(&policy, &compiler)
    }
}

/// Compile with an explicitly supplied CUE compiler policy. The target is validated first.
pub fn generate_with_policy(policy_json: &str, compiler_json: &str) -> Result<String, Error> {
    let policy = Policy::parse(policy_json)?;
    let compiler = Policy::parse(compiler_json)?;
    generate_driver(&policy, &compiler)
}

// The shell and module emitter remain native. Each expression gets a fresh shared
// compiler budget; the surrounding shell is not charged to that budget.
fn generate_driver(policy: &Policy, compiler: &dyn Driver) -> Result<String, Error> {
    let mut out = String::from(
        "// Generated lowering driver. Do not edit.\nuse bootstrap_executor::lowering::{self, Context, Driver, Error};\nuse serde_json::Value;\n\npub struct Compiled;\n\nimpl Driver for Compiled {\n    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {\n        match lowering::instruction_op(_node)? {\n",
    );
    for (op, expr) in &policy.cases {
        write!(out, "            {op:?}").expect("write to string");
        for (alias, _) in policy.aliases.iter().filter(|(_, target)| *target == op) {
            write!(out, " | {alias:?}").expect("write to string");
        }
        out.push_str(" => ");
        let node = serde_json::to_value(expr).context(JsonSnafu)?;
        out.push_str(&Context::run(&node, compiler)?);
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
    fn aliases_are_eager_and_preserve_original_op() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"base":{"op":"string","field":"op"}},"aliases":{"other":"base"}}"#,
        )?;
        assert_eq!(policy.lower(&json!({"op":"other"}))?, "\"other\"");
        for text in [
            r#"{"formatVersion":1,"cases":{"x":{"op":"text","value":""}},"aliases":{"a":"x","a":"x"}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"text","value":""}},"aliases":{"x":"x"}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"text","value":""}},"aliases":{"a":"b","b":"x"}}"#,
            r#"{"formatVersion":1,"cases":{"x":{"op":"text","value":""}},"aliases":null}"#,
        ] {
            assert!(Policy::parse(text).is_err());
        }
        Ok(())
    }

    #[test]
    fn identifier_validates_before_output_and_charges_one_frame()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 1024 * 1024 - 12,
                frames: 0,
            },
        };
        assert_eq!(
            ctx.identifier(&json!({"x":"bytePattern"}), "x", true)?,
            "byte_pattern"
        );
        assert_eq!(ctx.budget.steps, 1);
        assert_eq!(ctx.budget.frames, 0);
        assert!(
            ctx.identifier(&json!({"x":"bad-identifier"}), "x", true)
                .expect_err("invalid input first")
                .to_string()
                .contains("identifier")
        );
        assert_eq!(ctx.budget.frames, 0);
        assert!(ctx.identifier(&json!({"x":"x"}), "x", false).is_err());
        for value in ["", "0a", "é", "bad::path"] {
            assert!(
                Context::run(
                    &json!({"op":"x","v":value}),
                    &Policy::parse(
                        r#"{"formatVersion":1,"cases":{"x":{"op":"identifier","field":"v"}}}"#
                    )?
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn expression_serialization_projects_roles() -> Result<(), Box<dyn std::error::Error>> {
        let expr: super::Expr = serde_json::from_value(
            json!({"op":"concat","parts":[{"op":"call","path":"f","args":[{"op":"identifier","field":"op"}]},{"op":"select","field":"x","cases":{"z":{"op":"text","value":"z"},"a":{"op":"text","value":"a"}}}]}),
        )?;
        assert_eq!(
            serde_json::to_value(expr)?,
            json!({"op":"concat","parts":[{"op":"concatArgument","value":{"op":"call","path":"f","args":[{"op":"callArgument","value":{"op":"identifier","field":"op","snake":false}}]}},{"op":"concatArgument","value":{"op":"select","field":"x","cases":[{"op":"selectCase","tag":"a","value":{"op":"text","value":"a"}},{"op":"selectCase","tag":"z","value":{"op":"text","value":"z"}}]}}]})
        );
        Ok(())
    }

    #[test]
    fn bindings_are_lexical_and_branches_lazy() -> Result<(), Box<dyn std::error::Error>> {
        let expr = json!({"op":"bind","name":"saved","value":{"op":"string","field":"text"},"body":
            {"op":"ifEqual","left":{"op":"get","name":"saved"},"right":{"op":"string","field":"same"},
             "then":{"op":"concat","parts":[{"op":"get","name":"saved"},{"op":"text","value":"!"}]},
             "else":{"op":"ifTrue","field":"flag","then":{"op":"bind","name":"saved","value":{"op":"get","name":"saved"},"body":{"op":"get","name":"saved"}},"else":{"op":"text","value":"no"}}}});
        let text = json!({"formatVersion":1,"cases":{"root":expr}}).to_string();
        let policy = Policy::parse(&text)?;
        assert_eq!(
            policy.lower(&json!({"op":"root","text":"λ\"","same":"λ\""}))?,
            "\"λ\\\"\"!"
        );
        assert_eq!(
            policy.lower(&json!({"op":"root","text":"a","same":"b","flag":true}))?,
            "\"a\""
        );
        assert_eq!(
            policy.lower(&json!({"op":"root","text":"a","same":"b","flag":false}))?,
            "no"
        );
        assert!(
            policy
                .lower(&json!({"op":"root","text":"a","same":"b"}))
                .is_err()
        );
        let generated = super::generate(&text)?;
        assert!(generated.contains("ctx.bind(|ctx|"));
        assert!(generated.contains("|ctx, _render_saved|"));
        assert!(generated.contains("ctx.rendered(_render_saved)"));
        for expr in [
            json!({"op":"get","name":"missing"}),
            json!({"op":"bind","name":"x","value":{"op":"get","name":"x"},"body":{"op":"text","value":""}}),
            json!({"op":"ifTrue","field":"f","then":{"op":"text","value":""},"else":{"op":"get","name":"missing"}}),
            json!({"op":"bind","name":"X","value":{"op":"text","value":""},"body":{"op":"text","value":""}}),
            json!({"op":"bind","name":"a".repeat(65),"value":{"op":"text","value":""},"body":{"op":"text","value":""}}),
            json!({"op":"concat","parts":[{"op":"bind","name":"x","value":{"op":"text","value":""},"body":{"op":"text","value":""}},{"op":"get","name":"x"}]}),
        ] {
            assert!(
                Policy::parse(&json!({"formatVersion":1,"cases":{"x":expr}}).to_string()).is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn pattern_primitives_validate_sort_and_quote() -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"b":{"op":"bytePattern","field":"v"},"c":{"op":"charPattern","field":"v"}}}"#,
        )?;
        assert_eq!(
            policy.lower(&json!({"op":"b","v":[9,3,1,2,9,8,127]}))?,
            "1..=3 | 8 | 9 | 127"
        );
        assert_eq!(
            policy.lower(&json!({"op":"c","v":["λ","'","\n","\\","λ"]}))?,
            "'\\n' | '\\'' | '\\\\' | 'λ'"
        );
        for v in [
            json!([]),
            json!([128]),
            json!([-1]),
            json!([1.0]),
            json!([true]),
            json!(vec![1; 129]),
        ] {
            assert!(policy.lower(&json!({"op":"b","v":v})).is_err());
        }
        for v in [
            json!([]),
            json!([""]),
            json!(["ab"]),
            json!([1]),
            json!(vec!["x"; 129]),
        ] {
            assert!(policy.lower(&json!({"op":"c","v":v})).is_err());
        }
        let generated = super::generate(
            r#"{"formatVersion":1,"cases":{"b":{"op":"bytePattern","field":"v"},"c":{"op":"charPattern","field":"v"}}}"#,
        )?;
        assert!(generated.contains("ctx.byte_pattern(_node, \"v\")"));
        assert!(generated.contains("ctx.char_pattern(_node, \"v\")"));
        Ok(())
    }

    #[test]
    fn pattern_bytes_have_exact_caps_and_validate_before_writing()
    -> Result<(), Box<dyn std::error::Error>> {
        let policy = Policy::parse(
            r#"{"formatVersion":1,"cases":{"b":{"op":"bytePattern","field":"v"},"c":{"op":"charPattern","field":"v"}}}"#,
        )?;
        for (op, values, expected, invalid, message) in [
            (
                "b",
                json!([5, 4, 2, 0, 1]),
                "0..=2 | 4 | 5",
                json!([128]),
                "ASCII unsigned integers",
            ),
            (
                "c",
                json!(["λ", "'", "\n"]),
                "'\\n' | '\\'' | 'λ'",
                json!(["ab"]),
                "single scalar strings",
            ),
        ] {
            for remaining in [expected.len(), expected.len() - 1] {
                let mut ctx = Context {
                    budget: Budget {
                        steps: 0,
                        bytes: 1024 * 1024 - remaining,
                        frames: 0,
                    },
                };
                let result = ctx.dispatch(&json!({"op":op,"v":values}), 1, &policy);
                if remaining == expected.len() {
                    assert_eq!(result?, expected);
                } else {
                    assert_eq!(
                        result.expect_err("one byte short").to_string(),
                        "lowering: lowering exceeds 1 MiB rendered bytes"
                    );
                }
                assert_eq!(ctx.budget.frames, 0);
            }
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024,
                    frames: 0,
                },
            };
            assert!(
                ctx.dispatch(&json!({"op":op,"v":invalid}), 1, &policy)
                    .expect_err("validate before output")
                    .to_string()
                    .contains(message)
            );
            assert_eq!(ctx.budget.frames, 0);
        }
        Ok(())
    }

    #[test]
    fn new_primitives_share_budget_and_preserve_order() {
        for (remaining, success) in [(3, true), (2, false)] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            let result = ctx.bind(|ctx| ctx.text("x"), |ctx, x| ctx.rendered(x));
            // value, rendered copy and bind copy charge three bytes.
            assert_eq!(result.is_ok(), success);
            assert_eq!(ctx.budget.steps, 3);
            assert_eq!(ctx.budget.frames, 0);
        }
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 0,
            },
        };
        let result = ctx.if_equal(
            |ctx| ctx.text("a"),
            |ctx| ctx.unsigned(&json!({"n":false}), "n"),
            |_| panic!("branch"),
            |_| panic!("branch"),
        );
        assert!(
            result
                .expect_err("right evaluated")
                .to_string()
                .contains("unsigned integer")
        );
        assert_eq!(ctx.budget.frames, 0);
        let mut ctx = Context {
            budget: Budget {
                steps: 0,
                bytes: 1024 * 1024 - 2,
                frames: 0,
            },
        };
        assert!(ctx.char_pattern(&json!({"v":["λ"]}), "v").is_err());
        assert_eq!(ctx.budget.frames, 0);
        let mut ctx = Context {
            budget: Budget {
                steps: 4095,
                bytes: 0,
                frames: 0,
            },
        };
        assert!(
            ctx.if_true(
                &json!({"f":true}),
                "f",
                |ctx| ctx.text("x"),
                |_| panic!("branch")
            )
            .is_err()
        );
        assert_eq!(ctx.budget.frames, 0);
    }

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
    fn signed_join_and_zero_validate_and_render() -> Result<(), Box<dyn std::error::Error>> {
        let text = json!({"formatVersion":1,"cases":{
            "root":{"op":"join","field":"items","separator":"|\"\\\nλ"},
            "item":{"op":"ifZero","field":"count","then":{"op":"signed","field":"n"},"else":{"op":"text","value":"other"}}
        }}).to_string();
        let policy = Policy::parse(&text)?;
        assert_eq!(policy.lower(&json!({"op":"root","items":[]}))?, "");
        assert_eq!(
            policy.lower(&json!({"op":"root","items":[
                {"op":"item","count":0,"n":i64::MIN},
                {"op":"item","count":0,"n":i64::MAX},
                {"op":"item","count":1,"n":false}
            ]}))?,
            format!("{}|\"\\\nλ{}|\"\\\nλother", i64::MIN, i64::MAX)
        );
        for value in [json!(false), json!(-1), json!(1.5)] {
            assert!(policy.lower(&json!({"op":"item","count":value})).is_err());
        }
        for value in [json!(false), json!(u64::MAX), json!(1.5)] {
            assert!(
                policy
                    .lower(&json!({"op":"item","count":0,"n":value}))
                    .is_err()
            );
        }
        for size in [0, 129] {
            for op in ["signed", "join", "ifZero"] {
                let expr = match op {
                    "join" => json!({"op":op,"field":"x".repeat(size),"separator":""}),
                    "ifZero" => {
                        json!({"op":op,"field":"x".repeat(size),"then":{"op":"text","value":""},"else":{"op":"text","value":""}})
                    }
                    _ => json!({"op":op,"field":"x".repeat(size)}),
                };
                assert!(
                    Policy::parse(&json!({"formatVersion":1,"cases":{"x":expr}}).to_string())
                        .is_err()
                );
            }
        }
        assert!(Policy::parse(&json!({"formatVersion":1,"cases":{"x":{"op":"join","field":"a","separator":"λ".repeat(65)}}}).to_string()).is_err());
        assert!(Policy::parse(&json!({"formatVersion":1,"cases":{"x":{"op":"ifZero","field":"a","then":{"op":"text","value":""},"else":{"op":"signed","field":""}}}}).to_string()).is_err());
        let generated = super::generate(&text)?;
        assert!(generated.contains(r#"ctx.join(_node, "items", "|\"\\\nλ", _depth, self)"#));
        assert!(generated.contains("ctx.if_zero(_node, \"count\""));
        assert!(generated.contains("ctx.signed(_node, \"n\")"));
        Ok(())
    }

    #[test]
    fn join_and_zero_charge_and_order() -> Result<(), Box<dyn std::error::Error>> {
        let policy =
            Policy::parse(r#"{"formatVersion":1,"cases":{"item":{"op":"signed","field":"n"}}}"#)?;
        for (remaining, expected) in [
            (4, "lowering: field \"n\" must be a signed integer"),
            (3, "lowering: lowering exceeds 1 MiB rendered bytes"),
        ] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            let result = ctx.join(
                &json!({"a":[{"op":"item","n":1},{"op":"item","n":false}]}),
                "a",
                "||",
                1,
                &policy,
            );
            assert_eq!(result.expect_err("join error").to_string(), expected);
            assert_eq!(ctx.budget.frames, 0);
        }
        for (remaining, success) in [(2, true), (1, false)] {
            let mut ctx = Context {
                budget: Budget {
                    steps: 0,
                    bytes: 1024 * 1024 - remaining,
                    frames: 0,
                },
            };
            assert_eq!(
                ctx.if_zero(
                    &json!({"n":0}),
                    "n",
                    |ctx| ctx.text("x"),
                    |_| panic!("unselected")
                )
                .is_ok(),
                success
            );
            assert_eq!(ctx.budget.steps, 2);
            assert_eq!(ctx.budget.frames, 0);
        }
        let node = json!({"a":[{"op":"item","n":1},{"op":"item","n":2}]});
        let mut list = Context {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 0,
            },
        };
        let mut join = Context {
            budget: Budget {
                steps: 0,
                bytes: 0,
                frames: 0,
            },
        };
        assert_eq!(
            list.list(&node, "a", 1, &policy)?,
            join.join(&node, "a", "", 1, &policy)?
        );
        assert_eq!(
            (list.budget.steps, list.budget.bytes, list.budget.frames),
            (join.budget.steps, join.budget.bytes, join.budget.frames)
        );
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
