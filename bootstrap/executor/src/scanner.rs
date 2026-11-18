//! CUE scanner algorithms and compiler layout over native validation, flow facts,
//! reference execution and checked cursor/pattern primitives.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use snafu::{ResultExt, Snafu};

#[cfg(feature = "source")]
#[path = "../../generated/whitespace.rs"]
mod generated;

#[cfg(feature = "source")]
#[path = "../../generated/decimal.rs"]
mod generated_decimal;

#[cfg(feature = "source")]
#[path = "../../generated/integer-literal.rs"]
mod generated_integer_literal;

#[cfg(feature = "source")]
#[path = "../../generated/identifier.rs"]
mod generated_identifier;

#[cfg(feature = "source")]
#[path = "../../generated/after-operand.rs"]
mod generated_after_operand;
#[cfg(feature = "source")]
#[path = "../../generated/after-word-operator.rs"]
mod generated_after_word_operator;

#[cfg(feature = "source")]
#[path = "../../generated/group-open.rs"]
mod generated_group_open;

#[cfg(feature = "source")]
#[path = "../../generated/call-open.rs"]
mod generated_call_open;

#[cfg(feature = "source")]
#[path = "../../generated/group-close.rs"]
mod generated_group_close;

#[cfg(feature = "source")]
#[path = "../../generated/list-open.rs"]
mod generated_list_open;

#[cfg(feature = "source")]
#[path = "../../generated/list-close.rs"]
mod generated_list_close;

#[cfg(feature = "source")]
#[path = "../../generated/index-close.rs"]
mod generated_index_close;

#[cfg(feature = "source")]
#[path = "../../generated/list-tail.rs"]
mod generated_list_tail;

#[cfg(feature = "source")]
#[path = "../../generated/record-close.rs"]
mod generated_record_close;
#[cfg(feature = "source")]
#[path = "../../generated/record-colon.rs"]
mod generated_record_colon;
#[cfg(feature = "source")]
#[path = "../../generated/record-label.rs"]
mod generated_record_label;
#[cfg(feature = "source")]
#[path = "../../generated/selector-identifier.rs"]
mod generated_selector_identifier;
#[cfg(feature = "source")]
#[path = "../../generated/selector-open.rs"]
mod generated_selector_open;

#[cfg(feature = "source")]
#[path = "../../generated/record-open.rs"]
mod generated_record_open;
#[cfg(feature = "source")]
#[path = "../../generated/record-tail.rs"]
mod generated_record_tail;

#[cfg(feature = "source")]
#[path = "../../generated/bound-prefix.rs"]
mod generated_bound_prefix;

#[cfg(feature = "source")]
#[path = "../../generated/infix.rs"]
mod generated_infix;

#[cfg(feature = "source")]
#[path = "../../generated/negation-prefix.rs"]
mod generated_negation_prefix;
#[cfg(feature = "source")]
#[path = "../../generated/not-prefix.rs"]
mod generated_not_prefix;

#[cfg(feature = "source")]
#[path = "../../generated/string-literal.rs"]
mod generated_string_literal;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid scanner JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("{source}"))]
    Lowering { source: crate::lowering::Error },
    #[snafu(display("invalid scanner program: {reason}"))]
    Program { reason: &'static str },
    #[snafu(display("scanner offset is not a source character boundary"))]
    Offset,
    #[snafu(display("scanner cannot advance at EOF or over a non-ASCII byte"))]
    Advance,
    #[snafu(display("scanner cannot advance a character at EOF"))]
    AdvanceChar,
    #[snafu(display("scanner execution budget exhausted"))]
    Budget,
    #[snafu(display("source byte {offset}: {reason}"))]
    Rejected { offset: usize, reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scan {
    pub offset: usize,
    pub flag: bool,
}

/// Native boundary: validated cursor, checked advancement, one flag and fuel.
/// Each emitted/interpreted statement or loop test consumes one step.
pub struct Cursor<'a> {
    source: &'a str,
    offset: usize,
    flag: bool,
    fuel: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str, offset: usize) -> Result<Self, Error> {
        if !source.is_char_boundary(offset) {
            return Err(Error::Offset);
        }
        Ok(Self {
            source,
            offset,
            flag: false,
            fuel: (source.len() - offset)
                .saturating_add(1)
                .saturating_mul(128),
        })
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_at(0)
    }

    pub fn peek_at(&self, ahead: usize) -> Option<u8> {
        self.source
            .as_bytes()
            .get(self.offset.checked_add(ahead)?)
            .copied()
    }

    pub fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    pub fn advance_char(&mut self) -> Result<(), Error> {
        let character = self.peek_char().ok_or(Error::AdvanceChar)?;
        self.offset += character.len_utf8();
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        self.fuel = self.fuel.checked_sub(1).ok_or(Error::Budget)?;
        Ok(())
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        if !self.peek().is_some_and(|byte| byte.is_ascii()) {
            return Err(Error::Advance);
        }
        self.offset += 1;
        Ok(())
    }

    pub fn flag(&self) -> bool {
        self.flag
    }

    pub fn set_flag(&mut self, value: bool) {
        self.flag = value;
    }

    pub fn reject(&self, reason: &str) -> Result<(), Error> {
        Err(Error::Rejected {
            offset: self.offset,
            reason: reason.to_owned(),
        })
    }

    pub fn finish(self) -> Scan {
        Scan {
            offset: self.offset,
            flag: self.flag,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    body: Vec<Statement>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Predicate {
    #[serde(rename = "byteIn")]
    ByteIn {
        bytes: Vec<u8>,
        #[serde(default)]
        ahead: usize,
    },
    #[serde(rename = "charIn")]
    CharIn { characters: Vec<char> },
    #[serde(rename = "flagIs")]
    FlagIs { value: bool },
    #[serde(rename = "hasByte")]
    HasByte {},
    #[serde(rename = "and")]
    And {
        left: Box<Predicate>,
        right: Box<Predicate>,
    },
    #[serde(rename = "or")]
    Or {
        left: Box<Predicate>,
        right: Box<Predicate>,
    },
}

impl Predicate {
    fn validate(&self, depth: usize, nodes: &mut usize) -> Result<(), Error> {
        *nodes += 1;
        if *nodes > 128 {
            return ProgramSnafu {
                reason: "exceeds 128 predicate nodes",
            }
            .fail();
        }
        if depth > 16 {
            return ProgramSnafu {
                reason: "exceeds 16 predicate levels",
            }
            .fail();
        }
        match self {
            Self::And { left, right } | Self::Or { left, right } => {
                left.validate(depth + 1, nodes)?;
                right.validate(depth + 1, nodes)
            }
            Self::ByteIn { bytes, .. }
                if bytes.is_empty()
                    || bytes.len() > 128
                    || bytes.iter().any(|byte| !byte.is_ascii()) =>
            {
                ProgramSnafu {
                    reason: "byteIn requires 1..=128 ASCII bytes",
                }
                .fail()
            }
            Self::CharIn { characters } if characters.is_empty() || characters.len() > 128 => {
                ProgramSnafu {
                    reason: "charIn requires 1..=128 characters",
                }
                .fail()
            }
            Self::ByteIn { ahead, .. } if *ahead > 64 => ProgramSnafu {
                reason: "byteIn lookahead exceeds 64 bytes",
            }
            .fail(),
            _ => Ok(()),
        }
    }

    // Validation bounds pure tree work; only the surrounding statement charges fuel.
    fn test(&self, cursor: &Cursor<'_>) -> bool {
        match self {
            Self::ByteIn { bytes, ahead } => cursor
                .peek_at(*ahead)
                .is_some_and(|byte| bytes.contains(&byte)),
            Self::CharIn { characters } => cursor
                .peek_char()
                .is_some_and(|ch| characters.contains(&ch)),
            Self::FlagIs { value } => cursor.flag() == *value,
            Self::HasByte {} => cursor.peek().is_some(),
            Self::And { left, right } => left.test(cursor) && right.test(cursor),
            Self::Or { left, right } => left.test(cursor) || right.test(cursor),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Statement {
    #[serde(rename = "while")]
    While {
        condition: Predicate,
        body: Vec<Statement>,
    },
    #[serde(rename = "repeat")]
    Repeat { count: usize, body: Vec<Statement> },
    #[serde(rename = "if")]
    If {
        condition: Predicate,
        then: Vec<Statement>,
        #[serde(rename = "else", default)]
        otherwise: Vec<Statement>,
    },
    #[serde(rename = "advance")]
    Advance {},
    #[serde(rename = "advanceChar")]
    AdvanceChar {},
    #[serde(rename = "break")]
    Break {},
    #[serde(rename = "setFlag")]
    SetFlag { value: bool },
    #[serde(rename = "reject")]
    Reject { reason: String },
}

fn validate(
    body: &[Statement],
    depth: usize,
    loops: usize,
    nodes: &mut usize,
    predicate_nodes: &mut usize,
) -> Result<(), Error> {
    if depth > 16 {
        return ProgramSnafu {
            reason: "exceeds 16 nesting levels",
        }
        .fail();
    }
    for statement in body {
        *nodes += 1;
        if *nodes > 64 {
            return ProgramSnafu {
                reason: "exceeds 64 statements",
            }
            .fail();
        }
        match statement {
            Statement::While { condition, body } => {
                condition.validate(1, predicate_nodes)?;
                validate(body, depth + 1, loops + 1, nodes, predicate_nodes)?;
            }
            Statement::Repeat { count, body } => {
                if *count > 64 {
                    return ProgramSnafu {
                        reason: "repeat count exceeds 64",
                    }
                    .fail();
                }
                validate(body, depth + 1, loops + 1, nodes, predicate_nodes)?;
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate(1, predicate_nodes)?;
                validate(then, depth + 1, loops, nodes, predicate_nodes)?;
                validate(otherwise, depth + 1, loops, nodes, predicate_nodes)?;
            }
            Statement::Reject { reason } if reason.is_empty() => {
                return ProgramSnafu {
                    reason: "reject requires a nonempty reason",
                }
                .fail();
            }
            Statement::Break {} if loops == 0 => {
                return ProgramSnafu {
                    reason: "break requires an enclosing loop",
                }
                .fail();
            }
            Statement::Advance {}
            | Statement::AdvanceChar {}
            | Statement::Break {}
            | Statement::SetFlag { .. }
            | Statement::Reject { .. } => {}
        }
    }
    Ok(())
}

fn parse(input: &str) -> Result<Program, Error> {
    let program: Program = serde_json::from_str(input).context(JsonSnafu)?;
    if program.format_version != 1 {
        return ProgramSnafu {
            reason: "formatVersion 1 required",
        }
        .fail();
    }
    validate(&program.body, 0, 0, &mut 0, &mut 0)?;
    Ok(program)
}

fn execute(body: &[Statement], cursor: &mut Cursor<'_>) -> Result<bool, Error> {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                while {
                    cursor.step()?;
                    condition.test(cursor)
                } {
                    if execute(body, cursor)? {
                        break;
                    }
                }
            }
            Statement::Repeat { count, body } => {
                cursor.step()?;
                for _ in 0..*count {
                    cursor.step()?;
                    if execute(body, cursor)? {
                        break;
                    }
                }
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                cursor.step()?;
                if execute(
                    if condition.test(cursor) {
                        then
                    } else {
                        otherwise
                    },
                    cursor,
                )? {
                    return Ok(true);
                }
            }
            Statement::Advance {} => {
                cursor.step()?;
                cursor.advance()?;
            }
            Statement::AdvanceChar {} => {
                cursor.step()?;
                cursor.advance_char()?;
            }
            Statement::Break {} => {
                cursor.step()?;
                return Ok(true);
            }
            Statement::SetFlag { value } => {
                cursor.step()?;
                cursor.set_flag(*value);
            }
            Statement::Reject { reason } => {
                cursor.step()?;
                cursor.reject(reason)?;
            }
        }
    }
    Ok(false)
}

/// Reference execution of a validated JSON statement program.
pub fn interpret(input: &str, source: &str, offset: usize) -> Result<Scan, Error> {
    let program = parse(input)?;
    let mut cursor = Cursor::new(source, offset)?;
    execute(&program.body, &mut cursor)?;
    Ok(cursor.finish())
}

// True only when every path reaching the end of this sequence instead breaks
// the current loop. Nested loops consume their own breaks and may fall through.
fn always_breaks(body: &[Statement]) -> bool {
    body.iter().any(|statement| match statement {
        Statement::Break {} => true,
        Statement::If {
            then, otherwise, ..
        } => always_breaks(then) && always_breaks(otherwise),
        _ => false,
    })
}

// Nested loops consume their own breaks; only branches can expose one here.
fn may_break(body: &[Statement]) -> bool {
    body.iter().any(|statement| match statement {
        Statement::Break {} => true,
        Statement::If {
            then, otherwise, ..
        } => may_break(then) || may_break(otherwise),
        _ => false,
    })
}

fn predicate_node(predicate: &Predicate, inverse: bool) -> Result<Value, Error> {
    // Supply inversion as data at every level; CUE selects the operators.
    match predicate {
        Predicate::And { left, right } | Predicate::Or { left, right } => {
            let op = if matches!(predicate, Predicate::And { .. }) {
                "and"
            } else {
                "or"
            };
            return Ok(json!({
                "op": op,
                "inverse": inverse,
                "left": predicate_node(left, inverse)?,
                "right": predicate_node(right, inverse)?,
            }));
        }
        _ => {}
    }
    #[derive(Serialize)]
    struct Node<'a> {
        #[serde(flatten)]
        predicate: &'a Predicate,
        inverse: bool,
    }
    serde_json::to_value(Node { predicate, inverse }).context(JsonSnafu)
}

// Native flow facts: nearest-loop targets and unreachable suffixes, not source.
fn prepare(body: &[Statement], depth: usize, label: Option<usize>) -> Result<Vec<Value>, Error> {
    let mut nodes = Vec::new();
    for statement in body {
        nodes.push(match statement {
            Statement::While { condition, body } => {
                let once = always_breaks(body);
                json!({"op":"while","condition":predicate_node(condition,false)?,"once":once,"depth":depth,
                    "body":prepare(body,depth+1,if once {Some(depth)} else {None})?})
            }
            Statement::Repeat { count, body } => {
                let once = *count == 1 || always_breaks(body);
                let labeled = once && may_break(body);
                json!({"op":"repeat","count":count,"once":once,"labeled":labeled,"depth":depth,
                    "body":prepare(body,depth+1,if labeled {Some(depth)} else {None})?})
            }
            Statement::If { condition, then, otherwise } => {
                let then = prepare(then,depth,label)?;
                let otherwise = prepare(otherwise,depth,label)?;
                json!({"op":"if","condition":predicate_node(condition,false)?,"inverted":predicate_node(condition,true)?,"then":then,"else":otherwise})
            }
            Statement::Advance {} => json!({"op":"advance"}),
            Statement::AdvanceChar {} => json!({"op":"advanceChar"}),
            Statement::Break {} => json!({"op":"break","labeled":label.is_some(),"label":label.unwrap_or_default()}),
            Statement::SetFlag { value } => json!({"op":"setFlag","value":value}),
            Statement::Reject { reason } => json!({"op":"reject","reason":reason}),
        });
        if always_breaks(std::slice::from_ref(statement)) {
            break;
        }
    }
    Ok(nodes)
}

/// Compile through CUE-owned source/branch layout over checked native flow facts.
pub fn generate(input: &str) -> Result<String, Error> {
    #[cfg(feature = "compiled-lowering")]
    {
        lower(parse(input)?, &crate::compiled_scan_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        generate_with_policy(
            input,
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/scan-lowering.json"
            )),
        )
    }
}

pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, Error> {
    let program = parse(input)?;
    let policy =
        crate::lowering::Policy::parse(policy).map_err(|source| Error::Lowering { source })?;
    lower(program, &policy)
}

fn lower(program: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let node = json!({"op":"program","body":prepare(&program.body,0,None)?});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}

/// CUE-selected quoted string span; native conversion does not choose syntax.
#[cfg(feature = "source")]
pub fn string_literal(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_string_literal::scan(source, offset)
}

/// CUE-owned trivia/newline policy selected for word operators by parser control.
#[cfg(feature = "source")]
pub fn after_word_operator(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_after_word_operator::scan(source, offset)
}

/// Production whitespace scanner, compiled from the CUE statement program.
#[cfg(feature = "source")]
pub fn whitespace(source: &str, offset: usize) -> Result<Scan, Error> {
    generated::scan(source, offset)
}

/// Skip whitespace after an operand, rejecting a newline followed by more input.
#[cfg(feature = "source")]
pub fn after_operand(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_after_operand::scan(source, offset)
}

/// Recognize and consume an opening parenthesis; the flag records a match.
#[cfg(feature = "source")]
pub fn group_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_group_open::scan(source, offset)
}

/// Skip call-name trivia and require an opening parenthesis under CUE policy.
#[cfg(feature = "source")]
pub fn call_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_call_open::scan(source, offset)
}

/// Skip whitespace, require a closing parenthesis and consume it.
/// The result flag reports newlines in skipped whitespace, not token recognition.
#[cfg(feature = "source")]
pub fn group_close(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_group_close::scan(source, offset)
}

/// Optionally recognize an opening list token; the flag records recognition.
#[cfg(feature = "source")]
pub fn list_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_list_open::scan(source, offset)
}

/// Optionally recognize a closing list token; the flag records recognition.
#[cfg(feature = "source")]
pub fn list_close(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_list_close::scan(source, offset)
}

/// Run CUE-owned trivia and required closing-index recognition.
#[cfg(feature = "source")]
pub fn index_close(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_index_close::scan(source, offset)
}

/// Require a list delimiter; CUE reports comma as false and close as true.
#[cfg(feature = "source")]
pub fn list_tail(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_list_tail::scan(source, offset)
}

#[cfg(feature = "source")]
pub fn selector_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_selector_open::scan(source, offset)
}
#[cfg(feature = "source")]
pub fn selector_identifier(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_selector_identifier::scan(source, offset)
}

#[cfg(feature = "source")]
pub fn record_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_record_open::scan(source, offset)
}
#[cfg(feature = "source")]
pub fn record_close(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_record_close::scan(source, offset)
}
#[cfg(feature = "source")]
pub fn record_label(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_record_label::scan(source, offset)
}
#[cfg(feature = "source")]
pub fn record_colon(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_record_colon::scan(source, offset)
}
#[cfg(feature = "source")]
pub fn record_tail(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_record_tail::scan(source, offset)
}

/// Recognize a complete inclusive-bound prefix; no match leaves offset unchanged.
#[cfg(feature = "source")]
pub fn bound_prefix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_bound_prefix::scan(source, offset)
}

/// Recognize one byte from the bootstrap infix alphabet, independent of grammar lookup.
#[cfg(feature = "source")]
pub fn infix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_infix::scan(source, offset)
}

/// Validate decimal spelling (digits required, no leading zeroes), not magnitude.
/// The result flag records whether the optional minus was consumed.
#[cfg(feature = "source")]
pub fn decimal(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_decimal::scan(source, offset)
}

/// Validate the integer spelling selected by CUE, not its magnitude.
/// The result flag records whether the optional minus was consumed.
#[cfg(feature = "source")]
pub fn integer_literal(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_integer_literal::scan(source, offset)
}

/// Recognize minus before a non-digit; signed integer literals stay atomic.
#[cfg(feature = "source")]
pub fn negation_prefix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_negation_prefix::scan(source, offset)
}

#[cfg(feature = "source")]
pub fn not_prefix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_not_prefix::scan(source, offset)
}

/// Recognize an ASCII identifier; the flag indicates a valid first byte.
/// An invalid start leaves the cursor unchanged and the flag false.
#[cfg(feature = "source")]
pub fn identifier(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_identifier::scan(source, offset)
}

#[cfg(test)]
mod tests {
    use super::{Cursor, Error, Predicate, Statement, generate, interpret, validate};

    fn repeat(count: usize, body: Vec<Statement>) -> Statement {
        Statement::Repeat { count, body }
    }

    #[test]
    fn repeat_charges_entry_and_attempted_iterations_without_final_test() {
        for (count, body, steps) in [
            (0, vec![Statement::Advance {}], 1),
            (1, vec![], 2),
            (64, vec![], 65),
            (63, vec![Statement::SetFlag { value: true }], 127),
            (64, vec![Statement::Break {}, Statement::Advance {}], 3),
        ] {
            let mut cursor = Cursor::new("", 0).expect("EOF cursor");
            assert!(!super::execute(&[repeat(count, body)], &mut cursor).expect("repeat"));
            assert_eq!(cursor.fuel, 128 - steps);
            assert_eq!(cursor.offset, 0);
        }
        for source in ["", "x"] {
            let mut cursor = Cursor::new(source, 0).expect("cursor");
            let result = super::execute(
                &[repeat(64, vec![Statement::SetFlag { value: true }])],
                &mut cursor,
            );
            if source.is_empty() {
                assert!(matches!(result, Err(Error::Budget)));
                assert_eq!(cursor.fuel, 0);
            } else {
                assert!(!result.expect("129 steps fit 256 fuel"));
                assert_eq!(cursor.fuel, 256 - 129);
            }
        }
    }

    #[test]
    fn repeat_and_while_consume_only_their_nearest_break() {
        for inner in [
            repeat(64, vec![Statement::AdvanceChar {}, Statement::Break {}]),
            Statement::While {
                condition: Predicate::HasByte {},
                body: vec![Statement::AdvanceChar {}, Statement::Break {}],
            },
        ] {
            let mut cursor = Cursor::new("λ😀ab!", 0).expect("Unicode cursor");
            super::execute(
                &[
                    repeat(2, vec![inner, Statement::AdvanceChar {}]),
                    Statement::SetFlag { value: true },
                ],
                &mut cursor,
            )
            .expect("nearest inner break");
            assert_eq!(
                cursor.finish(),
                super::Scan {
                    offset: 8,
                    flag: true
                }
            );
        }
        let mut cursor = Cursor::new("λ😀!", 0).expect("Unicode cursor");
        super::execute(
            &[
                Statement::While {
                    condition: Predicate::HasByte {},
                    body: vec![
                        repeat(64, vec![Statement::AdvanceChar {}, Statement::Break {}]),
                        Statement::AdvanceChar {},
                        Statement::Break {},
                    ],
                },
                Statement::SetFlag { value: true },
            ],
            &mut cursor,
        )
        .expect("repeat inside while");
        assert_eq!(
            cursor.finish(),
            super::Scan {
                offset: 6,
                flag: true
            }
        );
    }

    #[test]
    fn repeat_metadata_uses_exact_nearest_loop_and_once_facts() {
        use serde_json::json;
        let body = vec![
            repeat(
                2,
                vec![
                    repeat(64, vec![Statement::Break {}, Statement::Advance {}]),
                    Statement::AdvanceChar {},
                ],
            ),
            Statement::SetFlag { value: true },
        ];
        assert!(!super::always_breaks(&body));
        assert_eq!(
            serde_json::Value::Array(super::prepare(&body, 0, None).expect("metadata")),
            json!([
                {"op":"repeat","count":2,"once":false,"labeled":false,"depth":0,"body":[
                    {"op":"repeat","count":64,"once":true,"labeled":true,"depth":1,"body":[
                        {"op":"break","labeled":true,"label":1}]},
                    {"op":"advanceChar"}]},
                {"op":"setFlag","value":true}
            ])
        );
        for count in [0, 1, 2, 64] {
            let nodes =
                super::prepare(&[repeat(count, vec![])], 0, None).expect("empty repeat metadata");
            assert_eq!(nodes[0]["once"], count == 1);
            assert_eq!(nodes[0]["labeled"], false);
        }
        for (body, labeled) in [
            (vec![Statement::SetFlag { value: true }], false),
            (
                vec![Statement::If {
                    condition: Predicate::HasByte {},
                    then: vec![Statement::Break {}],
                    otherwise: vec![],
                }],
                true,
            ),
            (vec![repeat(64, vec![Statement::Break {}])], false),
            (
                vec![Statement::While {
                    condition: Predicate::HasByte {},
                    body: vec![Statement::Break {}],
                }],
                false,
            ),
        ] {
            assert_eq!(super::may_break(&body), labeled);
            let nodes = super::prepare(&[repeat(1, body)], 0, None).expect("once label");
            assert_eq!(nodes[0]["once"], true);
            assert_eq!(nodes[0]["labeled"], labeled);
            if labeled {
                assert_eq!(nodes[0]["body"][0]["then"][0]["label"], 0);
            }
        }
        // An outer labeled block must not capture a break in an inner real loop.
        // On return from the inner scope, the outer label must be restored.
        for inner in [
            repeat(
                2,
                vec![Statement::If {
                    condition: Predicate::HasByte {},
                    then: vec![Statement::Break {}],
                    otherwise: vec![],
                }],
            ),
            Statement::While {
                condition: Predicate::HasByte {},
                body: vec![Statement::If {
                    condition: Predicate::HasByte {},
                    then: vec![Statement::Break {}],
                    otherwise: vec![],
                }],
            },
        ] {
            let nodes = super::prepare(&[repeat(1, vec![inner, Statement::Break {}])], 3, None)
                .expect("nested scope");
            assert_eq!(
                nodes[0]["body"][0]["body"][0]["then"][0],
                json!({"op":"break","labeled":false,"label":0})
            );
            assert_eq!(
                nodes[0]["body"][1],
                json!({"op":"break","labeled":true,"label":3})
            );
        }
        // Repeat consumes its break, so it cannot make its enclosing while once-only.
        let nodes = super::prepare(
            &[Statement::While {
                condition: Predicate::HasByte {},
                body: vec![repeat(1, vec![Statement::Break {}])],
            }],
            0,
            None,
        )
        .expect("while scope");
        assert_eq!(nodes[0]["once"], false);
    }

    #[test]
    fn zero_repeat_validates_full_body_before_policy_or_flow_pruning() {
        for invalid in [
            r#"{"op":"unknownTestKind"}"#,
            r#"{"op":"repeat","body":[]}"#,
            r#"{"op":"repeat","count":-1,"body":[]}"#,
            r#"{"op":"repeat","count":1.5,"body":[]}"#,
            r#"{"op":"repeat","count":65,"body":[]}"#,
            r#"{"op":"repeat","count":0,"count":1,"body":[]}"#,
            r#"{"op":"repeat","count":0}"#,
            r#"{"op":"repeat","count":0,"body":[],"extra":0}"#,
            r#"{"op":"reject","reason":""}"#,
            r#"{"op":"if","condition":{"op":"byteIn","bytes":[],"ahead":65},"then":[]}"#,
        ] {
            let input = format!(
                r#"{{"formatVersion":1,"body":[{{"op":"repeat","count":0,"body":[{{"op":"break"}},{invalid}]}}]}}"#
            );
            let expected = super::parse(&input)
                .err()
                .expect("invalid body")
                .to_string();
            assert_eq!(
                interpret(&input, "", 0)
                    .expect_err("validate before execution")
                    .to_string(),
                expected
            );
            assert_eq!(
                super::generate_with_policy(&input, "not JSON")
                    .expect_err("validate before policy")
                    .to_string(),
                expected
            );
        }
        let input = r#"{"formatVersion":1,"body":[{"op":"repeat","count":0,"body":[{"op":"break"}]},{"op":"break"}]}"#;
        assert!(
            super::parse(input)
                .err()
                .expect("scope not leaked")
                .to_string()
                .contains("break requires an enclosing loop")
        );
        let input = r#"{"formatVersion":1,"body":[{"op":"repeat","count":65,"body":[]}]}"#;
        assert_eq!(
            super::parse(input).err().expect("count cap").to_string(),
            "invalid scanner program: repeat count exceeds 64"
        );
    }

    #[test]
    fn repeat_body_counts_once_and_shares_all_structural_limits() {
        for count in [0, 64] {
            let mut body = vec![repeat(
                count,
                (0..63).map(|_| Statement::Advance {}).collect(),
            )];
            let mut nodes = 0;
            validate(&body, 0, 0, &mut nodes, &mut 0).expect("body counted once");
            assert_eq!(nodes, 64);
            body.push(Statement::Advance {});
            assert!(
                validate(&body, 0, 0, &mut 0, &mut 0)
                    .expect_err("shared statements")
                    .to_string()
                    .contains("64 statements")
            );
            let mut body = vec![
                repeat(count, vec![conditional(tree(7))]),
                conditional(Predicate::HasByte {}),
            ];
            let mut predicates = 0;
            validate(&body, 0, 0, &mut 0, &mut predicates).expect("128 nodes shared");
            assert_eq!(predicates, 128);
            body.push(conditional(Predicate::HasByte {}));
            assert!(
                validate(&body, 0, 0, &mut 0, &mut 0)
                    .expect_err("shared predicates")
                    .to_string()
                    .contains("128 predicate nodes")
            );
            assert!(
                validate(
                    &[repeat(count, vec![conditional(chain(17))])],
                    0,
                    0,
                    &mut 0,
                    &mut 0
                )
                .expect_err("predicate depth")
                .to_string()
                .contains("16 predicate levels")
            );
            let mut body = vec![];
            for _ in 0..16 {
                body = vec![repeat(count, body)];
            }
            validate(&body, 0, 0, &mut 0, &mut 0).expect("16 nested repeats");
            assert!(
                validate(&[repeat(count, body)], 0, 0, &mut 0, &mut 0)
                    .expect_err("nesting limit")
                    .to_string()
                    .contains("16 nesting levels")
            );
        }
    }

    fn tree(levels: usize) -> Predicate {
        if levels == 1 {
            Predicate::HasByte {}
        } else {
            Predicate::And {
                left: Box::new(tree(levels - 1)),
                right: Box::new(tree(levels - 1)),
            }
        }
    }

    fn chain(levels: usize) -> Predicate {
        if levels == 1 {
            Predicate::HasByte {}
        } else {
            Predicate::Or {
                left: Box::new(Predicate::HasByte {}),
                right: Box::new(chain(levels - 1)),
            }
        }
    }

    fn conditional(condition: Predicate) -> Statement {
        Statement::If {
            condition,
            then: vec![],
            otherwise: vec![],
        }
    }

    #[test]
    fn predicate_limits_are_independent_and_whole_program() {
        // 127-node full tree plus a separate leaf, not a per-condition cap.
        let body = vec![conditional(tree(7)), conditional(Predicate::HasByte {})];
        let mut predicates = 0;
        validate(&body, 0, 0, &mut 0, &mut predicates).expect("128 predicates");
        assert_eq!(predicates, 128);
        let body = vec![conditional(Predicate::And {
            left: Box::new(Predicate::HasByte {}),
            right: Box::new(tree(7)),
        })];
        assert_eq!(
            validate(&body, 0, 0, &mut 0, &mut 0)
                .expect_err("129 predicates")
                .to_string(),
            "invalid scanner program: exceeds 128 predicate nodes"
        );
        validate(&[conditional(chain(16))], 0, 0, &mut 0, &mut 0)
            .expect("16 predicate levels, 31 nodes");
        assert_eq!(
            validate(&[conditional(chain(17))], 0, 0, &mut 0, &mut 0)
                .expect_err("17 predicate levels")
                .to_string(),
            "invalid scanner program: exceeds 16 predicate levels"
        );
        // Count across while/if, both branches, and an unreachable suffix.
        let body = vec![
            Statement::While {
                condition: tree(7),
                body: vec![Statement::Break {}, conditional(Predicate::HasByte {})],
            },
            conditional(Predicate::HasByte {}),
        ];
        assert!(
            validate(&body, 0, 0, &mut 0, &mut 0)
                .expect_err("unreachable predicate counted")
                .to_string()
                .contains("exceeds 128 predicate nodes")
        );
        let body = vec![Statement::If {
            condition: Predicate::HasByte {},
            then: vec![conditional(tree(7))],
            otherwise: vec![conditional(Predicate::HasByte {})],
        }];
        assert!(
            validate(&body, 0, 0, &mut 0, &mut 0)
                .expect_err("unused branch counted")
                .to_string()
                .contains("exceeds 128 predicate nodes")
        );
    }

    #[test]
    fn composite_reads_are_pure_and_charge_only_the_surrounding_test() {
        for left in [false, true] {
            for right in [false, true] {
                let mut cursor = Cursor::new("λx", 0).expect("cursor");
                cursor.set_flag(true);
                let fuel = cursor.fuel;
                for (op, expected) in [("and", left && right), ("or", left || right)] {
                    let condition = serde_json::from_value(serde_json::json!({
                        "op":op, "left":{"op":"flagIs","value":left},
                        "right":{"op":"flagIs","value":right}
                    }))
                    .expect("predicate");
                    let before_test = cursor.fuel;
                    assert_eq!(Predicate::test(&condition, &cursor), expected);
                    assert_eq!(cursor.fuel, before_test);
                    super::execute(&[conditional(condition)], &mut cursor).expect("if");
                }
                assert_eq!(cursor.fuel, fuel - 2);
                assert_eq!(cursor.offset, 0);
                assert!(cursor.flag);
            }
        }
        let mut cursor = Cursor::new("", 0).expect("EOF");
        let fuel = cursor.fuel;
        super::execute(&[conditional(tree(7))], &mut cursor).expect("bounded tree");
        assert_eq!(cursor.fuel, fuel - 1);
        super::execute(
            &[Statement::While {
                condition: chain(16),
                body: vec![],
            }],
            &mut cursor,
        )
        .expect("false loop condition");
        assert_eq!(cursor.fuel, fuel - 2);
    }

    #[test]
    fn composite_metadata_recursively_preserves_desired_inversion() {
        use serde_json::json;
        let predicate: Predicate = serde_json::from_value(json!({
            "op":"and", "left":{"op":"hasByte"},
            "right":{"op":"or", "left":{"op":"byteIn","bytes":[97],"ahead":1},
                "right":{"op":"charIn","characters":["λ"]}}
        }))
        .expect("tree");
        for inverse in [false, true] {
            assert_eq!(
                super::predicate_node(&predicate, inverse).expect("metadata"),
                json!({
                    "op":"and", "inverse":inverse,
                    "left":{"op":"hasByte","inverse":inverse},
                    "right":{"op":"or","inverse":inverse,
                        "left":{"op":"byteIn","bytes":[97],"ahead":1,"inverse":inverse},
                        "right":{"op":"charIn","characters":["λ"],"inverse":inverse}}
                })
            );
            assert_eq!(
                super::predicate_node(&Predicate::FlagIs { value: false }, inverse)
                    .expect("leaf metadata"),
                json!({"op":"flagIs","value":false,"inverse":inverse})
            );
        }
    }

    #[test]
    fn composite_children_validate_eagerly_before_policy_or_execution() {
        for invalid in [
            r#"{"op":"byteIn","bytes":[]}"#,
            r#"{"op":"byteIn","bytes":[128]}"#,
            r#"{"op":"byteIn","bytes":[97],"ahead":65}"#,
            r#"{"op":"charIn","characters":[]}"#,
            r#"{"op":"charIn","characters":["ab"]}"#,
            r#"{"op":"hasByte","extra":true}"#,
            r#"{"op":"flagIs","value":true,"value":false}"#,
            r#"{"op":"and","left":{"op":"hasByte"}}"#,
            r#"{"op":"or","right":{"op":"hasByte"}}"#,
            r#"{"op":"and","left":{"op":"hasByte"},"right":{"op":"hasByte"},"extra":0}"#,
            r#"{"op":"or","left":{"op":"hasByte"},"left":{"op":"hasByte"},"right":{"op":"hasByte"}}"#,
            r#"{"op":"not","left":{"op":"hasByte"}}"#,
        ] {
            for op in ["and", "or"] {
                for invalid_left in [false, true] {
                    let valid = r#"{"op":"flagIs","value":true}"#;
                    let (left, right) = if invalid_left {
                        (invalid, valid)
                    } else {
                        (valid, invalid)
                    };
                    let input = format!(
                        r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{{"op":"{op}","left":{left},"right":{right}}},"then":[]}}]}}"#
                    );
                    let expected = super::parse(&input)
                        .err()
                        .expect("invalid child")
                        .to_string();
                    assert_eq!(
                        interpret(&input, "", 0)
                            .expect_err("eager execution validation")
                            .to_string(),
                        expected
                    );
                    assert_eq!(
                        super::generate_with_policy(&input, "not JSON")
                            .expect_err("target validation before malformed policy")
                            .to_string(),
                        expected
                    );
                }
            }
        }
        // Oversized classes still reject inside compositions.
        for child in [
            serde_json::json!({"op":"byteIn","bytes":vec![97;129]}),
            serde_json::json!({"op":"charIn","characters":vec!["λ";129]}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"body":[{"op":"if",
                "condition":{"op":"or","left":{"op":"hasByte"},"right":child},"then":[]}]})
            .to_string();
            assert!(super::parse(&input).is_err());
        }
    }

    #[test]
    fn unicode_cursor_preserves_boundaries_and_ascii_contract() {
        assert!(matches!(Cursor::new("λ", 1), Err(Error::Offset)));
        let mut cursor = Cursor::new("λ😀a", 0).expect("boundary");
        assert!(matches!(cursor.advance(), Err(Error::Advance)));
        assert_eq!(cursor.peek_char(), Some('λ'));
        cursor.advance_char().expect("lambda");
        cursor.advance_char().expect("emoji");
        assert_eq!(cursor.finish().offset, 6);
        let mut eof = Cursor::new("", 0).expect("EOF boundary");
        assert!(matches!(eof.advance_char(), Err(Error::AdvanceChar)));
    }

    #[test]
    fn char_predicates_validate_and_escape() {
        let program = r#"{"formatVersion":1,"body":[{"op":"if","condition":{"op":"charIn","characters":["λ","\n","'","λ"]},"then":[{"op":"advanceChar"}]}]}"#;
        assert_eq!(interpret(program, "λ!", 0).expect("scan").offset, 2);
        let code = generate(program).expect("generate");
        assert!(code.contains("Some('\\n' | '\\'' | 'λ')"));
        for values in [r#"[]"#, r#"[""]"#, r#"["ab"]"#] {
            let invalid = format!(
                r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{{"op":"charIn","characters":{values}}},"then":[]}}]}}"#
            );
            assert!(generate(&invalid).is_err());
        }
        let values = vec!["x"; 129];
        let invalid = format!(
            r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{{"op":"charIn","characters":{}}},"then":[]}}]}}"#,
            serde_json::to_string(&values).expect("serialize")
        );
        assert!(generate(&invalid).is_err());
    }

    #[test]
    fn break_is_scoped_and_unreachable_suffixes_are_not_emitted() {
        let invalid = r#"{"formatVersion":1,"body":[{"op":"if","condition":{"op":"hasByte"},"then":[{"op":"break"}]}]}"#;
        assert!(generate(invalid).is_err());
        let program = r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"break"},{"op":"advance"}]},{"op":"if","condition":{"op":"charIn","characters":["λ"]},"then":[{"op":"break"}],"else":[{"op":"break"}]},{"op":"advance"}]},{"op":"setFlag","value":true}]}"#;
        let result = interpret(program, "λ", 0).expect("scan");
        assert_eq!(result.offset, 0);
        assert!(result.flag);
        let code = generate(program).expect("generate");
        assert!(!code.contains("cursor.advance()?"));
        assert!(code.contains("cursor.set_flag(true)"));
        assert!(!code.contains("while "));
        assert!(code.contains("break 'scan0;"));
        assert!(code.contains("break 'scan1;"));
        assert!(!code.contains("cursor.peek_char()")); // identical branches
        assert_eq!(code.matches("cursor.step()?").count(), 6);
        let conditional = r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"if","condition":{"op":"flagIs","value":true},"then":[{"op":"break"}]},{"op":"advance"}]}]}"#;
        let code = generate(conditional).expect("generate regular loop");
        assert!(code.contains("while "));
        assert!(code.contains("break;"));
        let looping =
            r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[]}]}"#;
        assert!(matches!(interpret(looping, "x", 0), Err(Error::Budget)));
    }

    #[test]
    fn byte_sets_compact_contiguous_runs_without_changing_sparse_patterns() {
        for (bytes, expected) in [
            (
                vec![13, 0, 1, 2, 4, 5, 10, 11, 12, 13],
                "matches!(cursor.peek(), Some(0..=2 | 4 | 5 | 10..=13))",
            ),
            (
                vec![32, 9, 10, 13],
                "matches!(cursor.peek(), Some(9 | 10 | 13 | 32))",
            ),
        ] {
            let input=serde_json::json!({"formatVersion":1,"body":[{"op":"if","condition":{"op":"byteIn","bytes":bytes},"then":[{"op":"advance"}]}]}).to_string();
            assert!(
                generate(&input)
                    .expect("CUE predicate layout")
                    .contains(expected)
            );
        }
    }

    #[test]
    fn statement_and_nesting_limits_are_checked_before_execution_or_emission() {
        let mut body: Vec<_> = (0..64).map(|_| Statement::Advance {}).collect();
        validate(&body, 0, 0, &mut 0, &mut 0).expect("64 statements allowed");
        body.push(Statement::Advance {});
        assert!(
            validate(&body, 0, 0, &mut 0, &mut 0)
                .expect_err("statement cap")
                .to_string()
                .contains("64 statements")
        );
        let mut body = vec![];
        for _ in 0..16 {
            body = vec![Statement::While {
                condition: Predicate::ByteIn {
                    bytes: vec![32],
                    ahead: 0,
                },
                body,
            }];
        }
        validate(&body, 0, 0, &mut 0, &mut 0).expect("16 nesting levels allowed");
        body = vec![Statement::While {
            condition: Predicate::ByteIn {
                bytes: vec![32],
                ahead: 0,
            },
            body,
        }];
        assert!(
            validate(&body, 0, 0, &mut 0, &mut 0)
                .expect_err("nesting cap")
                .to_string()
                .contains("16 nesting levels")
        );
    }
}
