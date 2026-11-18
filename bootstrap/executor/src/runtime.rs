//! Value primitives shared by interpreted and generated evaluators.
//! Scalar operations dispatch CUE-generated control over native kernels.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value as JsonValue;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

mod conjunction;
pub use conjunction::{ConjunctionDriver, ConjunctionFrame, Unifier, conjunction_with_driver};
#[cfg(feature = "semantics")]
pub use conjunction::{conjunction, conjunction_interpreted};

#[cfg(feature = "semantics")]
#[path = "../../generated/unify.rs"]
mod generated_unification;

#[cfg(feature = "semantics")]
#[path = "../../generated/difference-control.rs"]
mod generated_difference;
#[cfg(feature = "semantics")]
#[path = "../../generated/divide-control.rs"]
mod generated_divide;
#[cfg(feature = "semantics")]
#[path = "../../generated/modulo-control.rs"]
mod generated_modulo;
#[cfg(feature = "semantics")]
#[path = "../../generated/negate-control.rs"]
mod generated_negate;
#[cfg(feature = "semantics")]
#[path = "../../generated/product-control.rs"]
mod generated_product;
#[cfg(feature = "semantics")]
#[path = "../../generated/quotient-control.rs"]
mod generated_quotient;
#[cfg(feature = "semantics")]
#[path = "../../generated/remainder-control.rs"]
mod generated_remainder;
#[cfg(feature = "semantics")]
#[path = "../../generated/sum.rs"]
mod generated_sum;

#[cfg(feature = "semantics")]
#[path = "../../generated/string-length-control.rs"]
mod generated_string_length;

#[cfg(feature = "semantics")]
#[path = "../../generated/list-length-control.rs"]
mod generated_list_length;

#[cfg(feature = "semantics")]
#[path = "../../generated/length-control.rs"]
mod generated_length;

#[cfg(feature = "semantics")]
#[path = "../../generated/index.rs"]
mod generated_index;

#[cfg(feature = "semantics")]
#[path = "../../generated/list-index.rs"]
mod generated_list_index;

#[cfg(feature = "semantics")]
#[path = "../../generated/list-slice.rs"]
mod generated_list_slice;

#[cfg(feature = "semantics")]
#[path = "../../generated/record-index.rs"]
mod generated_record_index;
#[cfg(feature = "semantics")]
#[path = "../../generated/record-length.rs"]
mod generated_record_length;

#[cfg(feature = "semantics")]
#[path = "../../generated/equal.rs"]
mod generated_equal;

#[cfg(feature = "semantics")]
#[path = "../../generated/not-equal.rs"]
mod generated_not_equal;

#[cfg(feature = "semantics")]
#[path = "../../generated/greater.rs"]
mod generated_greater;
#[cfg(feature = "semantics")]
#[path = "../../generated/greater-equal.rs"]
mod generated_greater_equal;
#[cfg(feature = "semantics")]
#[path = "../../generated/integer-output.rs"]
mod generated_integer_output;
#[cfg(feature = "semantics")]
#[path = "../../generated/json-output.rs"]
mod generated_json_output;
#[cfg(feature = "semantics")]
#[path = "../../generated/less.rs"]
mod generated_less;
#[cfg(feature = "semantics")]
#[path = "../../generated/less-equal.rs"]
mod generated_less_equal;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-and.rs"]
mod generated_logical_and;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-not.rs"]
mod generated_logical_not;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-or.rs"]
mod generated_logical_or;
#[cfg(feature = "semantics")]
#[path = "../../generated/lower-bound.rs"]
mod generated_lower_bound;
#[cfg(feature = "semantics")]
#[path = "../../generated/lower-exclusive-bound.rs"]
mod generated_lower_exclusive_bound;
#[cfg(feature = "semantics")]
#[path = "../../generated/upper-bound.rs"]
mod generated_upper_bound;
#[cfg(feature = "semantics")]
#[path = "../../generated/upper-exclusive-bound.rs"]
mod generated_upper_exclusive_bound;

/// Per-invocation AST entry budget, not a bound on all evaluator work.
/// Recursive entries must reuse the callback's context.
#[derive(Debug)]
pub struct EvaluationContext {
    remaining: usize,
    depth: usize,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            remaining: 4096,
            depth: 0,
        }
    }
}

impl EvaluationContext {
    /// Charge one AST entry, restoring active depth on success or Result error.
    /// Accepted entries consume a visit even on error; rejected entries do not.
    pub fn visit<T>(
        &mut self,
        evaluate: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        if self.remaining == 0 {
            return Err(Error::EvaluationVisits);
        }
        if self.depth >= 64 {
            return Err(Error::EvaluationDepth);
        }
        self.remaining -= 1;
        self.depth += 1;
        let result = evaluate(self);
        self.depth -= 1;
        result
    }
}

#[derive(Clone, Copy)]
enum ListStage {
    Start,
    Ready(usize),
    Evaluated(usize),
    Appended(usize),
    Exhausted,
    Done,
}

/// Checked list protocol kernel; traversal and admission policy belong to control.
pub struct ListFrame<'a> {
    items: &'a [JsonValue],
    context: &'a mut EvaluationContext,
    evaluate: &'a mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    stage: ListStage,
    candidate: Option<Value>,
    buffer: Vec<Value>,
    shape: ValueShape,
    result: Option<Value>,
    fuel: usize,
}

impl ListFrame<'_> {
    pub fn step(&mut self) -> Result<(), Error> {
        if self.fuel == 0 {
            return Err(Error::ListControlBudget);
        }
        self.fuel -= 1;
        Ok(())
    }

    fn invalid<T>(reason: &'static str) -> Result<T, Error> {
        Err(Error::ListState { reason })
    }

    fn select(&mut self, index: Option<usize>) -> Result<(), Error> {
        if !matches!(self.stage, ListStage::Start) {
            return Self::invalid("list cursor already initialized");
        }
        self.stage = index.map_or(ListStage::Exhausted, ListStage::Ready);
        Ok(())
    }

    pub fn first_item(&mut self) -> Result<(), Error> {
        self.select(if self.items.is_empty() { None } else { Some(0) })
    }

    pub fn last_item(&mut self) -> Result<(), Error> {
        self.select(self.items.len().checked_sub(1))
    }

    pub fn has_item(&self) -> Result<bool, Error> {
        match self.stage {
            ListStage::Ready(_) | ListStage::Evaluated(_) | ListStage::Appended(_) => Ok(true),
            ListStage::Exhausted => Ok(false),
            _ => Self::invalid("list cursor is not active"),
        }
    }

    pub fn evaluate_item(&mut self) -> Result<(), Error> {
        let ListStage::Ready(index) = self.stage else {
            return Self::invalid("item evaluation requires ready cursor");
        };
        let item = self.items.get(index).ok_or(Error::ListState {
            reason: "item index out of bounds",
        })?;
        self.candidate = Some((self.evaluate)(item, self.context)?);
        self.stage = ListStage::Evaluated(index);
        Ok(())
    }

    pub fn item_kind_is(&self, kind: crate::unification::Kind) -> Result<bool, Error> {
        if !matches!(self.stage, ListStage::Evaluated(_)) {
            return Self::invalid("item kind requires evaluated candidate");
        }
        self.candidate
            .as_ref()
            .map(|value| crate::unification::kind_is(value, kind))
            .ok_or(Error::ListState {
                reason: "missing item candidate",
            })
    }

    pub fn append_item(&mut self) -> Result<(), Error> {
        let ListStage::Evaluated(index) = self.stage else {
            return Self::invalid("append requires evaluated candidate");
        };
        if self.buffer.len() >= 64 {
            return Err(Error::ListCapacity);
        }
        let value = self.candidate.as_ref().ok_or(Error::ListState {
            reason: "missing item candidate",
        })?;
        let shape = self.shape.list_include(value, index)?;
        let value = self.candidate.take().ok_or(Error::ListState {
            reason: "missing item candidate",
        })?;
        self.buffer.push(value);
        self.shape = shape;
        self.stage = ListStage::Appended(index);
        Ok(())
    }

    fn advance(&mut self, forward: bool) -> Result<(), Error> {
        let ListStage::Appended(index) = self.stage else {
            return Self::invalid("movement requires appended item");
        };
        let next = if forward {
            index
                .checked_add(1)
                .filter(|index| *index < self.items.len())
        } else {
            index.checked_sub(1)
        };
        self.stage = next.map_or(ListStage::Exhausted, ListStage::Ready);
        Ok(())
    }

    pub fn next_item(&mut self) -> Result<(), Error> {
        self.advance(true)
    }
    pub fn previous_item(&mut self) -> Result<(), Error> {
        self.advance(false)
    }

    pub fn reject_item(&mut self) -> Result<(), Error> {
        match self.stage {
            ListStage::Evaluated(index) => Err(Error::ListElement { index }),
            _ => Self::invalid("rejection requires evaluated candidate"),
        }
    }

    pub fn finish_list(&mut self) -> Result<(), Error> {
        if !matches!(self.stage, ListStage::Exhausted) || self.buffer.len() != self.items.len() {
            return Self::invalid("finish requires exhausted cursor and complete buffer");
        }
        self.result = Some(Value::List(ClosedList::new(std::mem::take(
            &mut self.buffer,
        ))?));
        self.stage = ListStage::Done;
        Ok(())
    }
}

pub type ListDriver = fn(&mut ListFrame<'_>) -> Result<(), Error>;

/// Run a trusted driver without charging a parent AST visit; callbacks own child entry.
pub fn list_expression_with_driver(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    driver: ListDriver,
) -> Result<Value, Error> {
    let items = field(node, name)?.as_array().ok_or_else(|| Error::Array {
        field: name.to_string(),
    })?;
    if items.len() > 64 {
        return Err(Error::ListCapacity);
    }
    let mut frame = ListFrame {
        items,
        context,
        evaluate,
        stage: ListStage::Start,
        candidate: None,
        buffer: Vec::new(),
        shape: ValueShape::EMPTY,
        result: None,
        fuel: 4096,
    };
    driver(&mut frame)?;
    if !matches!(frame.stage, ListStage::Done) {
        return Err(Error::ListState {
            reason: "unfinished list result",
        });
    }
    frame.result.ok_or(Error::ListState {
        reason: "unfinished list result",
    })
}

#[cfg(feature = "semantics")]
#[path = "../../generated/list-control.rs"]
mod generated_list_control;

#[cfg(feature = "semantics")]
pub fn list_expression(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    list_expression_with_driver(
        node,
        name,
        context,
        &mut evaluate,
        generated_list_control::execute,
    )
}

#[cfg(feature = "semantics")]
pub fn list_expression_interpreted(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    list_expression_with_driver(node, name, context, &mut evaluate, |frame| {
        crate::pratt::interpret_list(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/list-control.json"
            )),
            frame,
        )
    })
}

#[derive(Clone, Copy)]
enum RecordStage {
    Start,
    Ready(usize),
    Evaluated(usize),
    Appended(usize),
    Exhausted,
    Done,
}

/// Checked record protocol kernel; CUE controls traversal and child admission.
pub struct RecordFrame<'a> {
    items: &'a [JsonValue],
    context: &'a mut EvaluationContext,
    evaluate: &'a mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    stage: RecordStage,
    name: Option<String>,
    candidate: Option<Value>,
    values: BTreeMap<String, Value>,
    shape: ValueShape,
    result: Option<Value>,
    fuel: usize,
}

impl RecordFrame<'_> {
    pub fn step(&mut self) -> Result<(), Error> {
        if self.fuel == 0 {
            return Err(Error::RecordControlBudget);
        }
        self.fuel -= 1;
        Ok(())
    }

    fn invalid<T>(reason: &'static str) -> Result<T, Error> {
        Err(Error::RecordState { reason })
    }

    fn select(&mut self, index: Option<usize>) -> Result<(), Error> {
        if !matches!(self.stage, RecordStage::Start) {
            return Self::invalid("record cursor already initialized");
        }
        self.stage = index.map_or(RecordStage::Exhausted, RecordStage::Ready);
        Ok(())
    }

    pub fn first_item(&mut self) -> Result<(), Error> {
        self.select(if self.items.is_empty() { None } else { Some(0) })
    }

    pub fn last_item(&mut self) -> Result<(), Error> {
        self.select(self.items.len().checked_sub(1))
    }

    pub fn has_item(&self) -> Result<bool, Error> {
        match self.stage {
            RecordStage::Ready(_) | RecordStage::Evaluated(_) | RecordStage::Appended(_) => {
                Ok(true)
            }
            RecordStage::Exhausted => Ok(false),
            _ => Self::invalid("record cursor is not active"),
        }
    }

    pub fn evaluate_item(&mut self) -> Result<(), Error> {
        let RecordStage::Ready(index) = self.stage else {
            return Self::invalid("item evaluation requires ready cursor");
        };
        let item = self.items.get(index).ok_or(Error::RecordState {
            reason: "item index out of bounds",
        })?;
        let name = field(item, "name")?.as_str().ok_or_else(|| Error::Text {
            field: "name".to_owned(),
        })?;
        record_name(&self.values, name, index)?;
        let value = (self.evaluate)(field(item, "value")?, self.context)?;
        self.name = Some(name.to_owned());
        self.candidate = Some(value);
        self.stage = RecordStage::Evaluated(index);
        Ok(())
    }

    pub fn item_kind_is(&self, kind: crate::unification::Kind) -> Result<bool, Error> {
        if !matches!(self.stage, RecordStage::Evaluated(_)) {
            return Self::invalid("item kind requires evaluated candidate");
        }
        self.candidate
            .as_ref()
            .map(|value| crate::unification::kind_is(value, kind))
            .ok_or(Error::RecordState {
                reason: "missing item candidate",
            })
    }

    pub fn append_item(&mut self) -> Result<(), Error> {
        let RecordStage::Evaluated(index) = self.stage else {
            return Self::invalid("append requires evaluated candidate");
        };
        record_capacity(self.values.len().saturating_add(1))?;
        let name = self.name.as_ref().ok_or(Error::RecordState {
            reason: "missing item name",
        })?;
        record_name(&self.values, name, index)?;
        let value = self.candidate.as_ref().ok_or(Error::RecordState {
            reason: "missing item candidate",
        })?;
        let shape = self.shape.record_include(value, name)?;
        // All prospective checks precede taking either pending slot.
        let name = self.name.take().ok_or(Error::RecordState {
            reason: "missing item name",
        })?;
        let value = self.candidate.take().ok_or(Error::RecordState {
            reason: "missing item candidate",
        })?;
        self.values.insert(name, value);
        self.shape = shape;
        self.stage = RecordStage::Appended(index);
        Ok(())
    }

    fn advance(&mut self, forward: bool) -> Result<(), Error> {
        let RecordStage::Appended(index) = self.stage else {
            return Self::invalid("movement requires appended item");
        };
        let next = if forward {
            index
                .checked_add(1)
                .filter(|index| *index < self.items.len())
        } else {
            index.checked_sub(1)
        };
        self.stage = next.map_or(RecordStage::Exhausted, RecordStage::Ready);
        Ok(())
    }

    pub fn next_item(&mut self) -> Result<(), Error> {
        self.advance(true)
    }

    pub fn previous_item(&mut self) -> Result<(), Error> {
        self.advance(false)
    }

    pub fn reject_item(&mut self) -> Result<(), Error> {
        if !matches!(self.stage, RecordStage::Evaluated(_)) {
            return Self::invalid("rejection requires evaluated candidate");
        }
        let name = self.name.as_ref().ok_or(Error::RecordState {
            reason: "missing item name",
        })?;
        Err(Error::RecordElement {
            field: name.clone(),
        })
    }

    pub fn finish_record(&mut self) -> Result<(), Error> {
        if !matches!(self.stage, RecordStage::Exhausted) || self.values.len() != self.items.len() {
            return Self::invalid("finish requires exhausted cursor and complete map");
        }
        let fields = std::mem::take(&mut self.values).into_iter().collect();
        self.result = Some(Value::Record(ClosedRecord::new(fields)?));
        self.stage = RecordStage::Done;
        Ok(())
    }
}

pub type RecordDriver = fn(&mut RecordFrame<'_>) -> Result<(), Error>;

/// Run trusted control without charging a parent AST visit; callbacks own child entry.
pub fn record_expression_with_driver(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    driver: RecordDriver,
) -> Result<Value, Error> {
    let items = field(node, name)?.as_array().ok_or_else(|| Error::Array {
        field: name.to_owned(),
    })?;
    record_capacity(items.len())?;
    let mut frame = RecordFrame {
        items,
        context,
        evaluate,
        stage: RecordStage::Start,
        name: None,
        candidate: None,
        values: BTreeMap::new(),
        shape: ValueShape::EMPTY,
        result: None,
        fuel: 4096,
    };
    driver(&mut frame)?;
    if !matches!(frame.stage, RecordStage::Done) {
        return Err(Error::RecordState {
            reason: "unfinished record result",
        });
    }
    frame.result.ok_or(Error::RecordState {
        reason: "unfinished record result",
    })
}

#[cfg(feature = "semantics")]
#[path = "../../generated/record-control.rs"]
mod generated_record_control;

#[cfg(feature = "semantics")]
pub fn record_expression(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    record_expression_with_driver(
        node,
        name,
        context,
        &mut evaluate,
        generated_record_control::execute,
    )
}

#[cfg(feature = "semantics")]
pub fn record_expression_interpreted(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    record_expression_with_driver(node, name, context, &mut evaluate, |frame| {
        crate::pratt::interpret_record(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/record-control.json"
            )),
            frame,
        )
    })
}

/// Certified logical shape, counting shared children once per occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValueShape {
    depth: usize,
    nodes: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum ShapeError {
    Element,
    Depth,
    Nodes,
}

impl ValueShape {
    const EMPTY: Self = Self { depth: 1, nodes: 1 };

    /// Compute a prospective parent shape using only completed child certificates.
    fn include(self, value: &Value) -> Result<Self, ShapeError> {
        let child = match value {
            Value::Null | Value::Integer(_) | Value::Boolean(_) | Value::Text(_) => Self::EMPTY,
            Value::List(list) => list.shape,
            Value::Record(record) => record.shape,
            _ => return Err(ShapeError::Element),
        };
        // Both operands are certified bounded shapes, so these sums cannot overflow.
        let depth = self.depth.max(1 + child.depth);
        if depth > 32 {
            return Err(ShapeError::Depth);
        }
        let nodes = self.nodes + child.nodes;
        if nodes > 4096 {
            return Err(ShapeError::Nodes);
        }
        Ok(Self { depth, nodes })
    }

    fn list_include(self, value: &Value, index: usize) -> Result<Self, Error> {
        self.include(value).map_err(|error| match error {
            ShapeError::Element => Error::ListElement { index },
            ShapeError::Depth => Error::ListDepth,
            ShapeError::Nodes => Error::ListNodes,
        })
    }

    fn record_include(self, value: &Value, name: &str) -> Result<Self, Error> {
        self.include(value).map_err(|error| match error {
            ShapeError::Element => Error::RecordElement {
                field: name.to_owned(),
            },
            ShapeError::Depth => Error::RecordDepth,
            ShapeError::Nodes => Error::RecordNodes,
        })
    }
}

/// Immutable, shared closed list with bounded width and logical depth/node count.
/// Children are completed immutable collections, so shared ownership forms a DAG.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosedList {
    values: Arc<[Value]>,
    shape: ValueShape,
}

impl ClosedList {
    /// Validate capacity first, then each concrete child and prospective parent shape.
    pub fn new(values: Vec<Value>) -> Result<Self, Error> {
        if values.len() > 64 {
            return Err(Error::ListCapacity);
        }
        let mut shape = ValueShape::EMPTY;
        for (index, value) in values.iter().enumerate() {
            shape = shape.list_include(value, index)?;
        }
        Ok(Self {
            values: Arc::from(values),
            shape,
        })
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Immutable shared record; completed children form an acyclic value DAG.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosedRecord {
    values: Arc<BTreeMap<String, Value>>,
    shape: ValueShape,
}

impl ClosedRecord {
    /// Check capacity, then each name, duplication, child type, depth and nodes.
    pub fn new(fields: Vec<(String, Value)>) -> Result<Self, Error> {
        record_capacity(fields.len())?;
        let mut values = BTreeMap::new();
        let mut shape = ValueShape::EMPTY;
        for (index, (name, value)) in fields.into_iter().enumerate() {
            record_name(&values, &name, index)?;
            shape = shape.record_include(&value, &name)?;
            values.insert(name, value);
        }
        Ok(Self {
            values: Arc::new(values),
            shape,
        })
    }

    pub fn as_map(&self) -> &BTreeMap<String, Value> {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

fn record_capacity(count: usize) -> Result<(), Error> {
    if count > 64 {
        return Err(Error::RecordCapacity);
    }
    Ok(())
}

fn record_name(values: &BTreeMap<String, Value>, name: &str, index: usize) -> Result<(), Error> {
    if name.is_empty() || name.len() > 64 {
        return Err(Error::RecordName { index });
    }
    if values.contains_key(name) {
        return Err(Error::RecordDuplicate {
            field: name.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod mixed_tests;
#[cfg(test)]
mod nested_record_tests;
#[cfg(test)]
mod nested_tests;
#[cfg(test)]
mod record_expression_tests;
#[cfg(test)]
mod record_tests;

/// Owned value scaffold; abstract types are not conflicting values.
/// Concrete integers are still restricted to the bootstrap i64 domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    Top,
    Integer(i64),
    Int,
    Boolean(bool),
    Bool,
    Text(Arc<str>),
    String,
    List(ClosedList),
    Record(ClosedRecord),
    // Missing endpoints are infinite, not i64::MIN / i64::MAX. Even equal
    // endpoints remain abstract until unified with a concrete candidate.
    Bounds {
        lower: Option<i64>,
        upper: Option<i64>,
    },
}

pub fn program(input: &str) -> Result<JsonValue, Error> {
    let program: Program = serde_json::from_str(input).context(JsonSnafu {
        document: "program",
    })?;
    if program.format_version != 1 {
        return VersionSnafu {
            document: "program",
            version: program.format_version,
        }
        .fail();
    }
    Ok(program.entry)
}

pub fn kind(node: &JsonValue) -> Result<&str, Error> {
    node.get("kind")
        .and_then(JsonValue::as_str)
        .ok_or(Error::Kind)
}

pub fn field<'a>(node: &'a JsonValue, name: &str) -> Result<&'a JsonValue, Error> {
    node.get(name).ok_or_else(|| Error::MissingField {
        field: name.to_string(),
    })
}

pub fn integer(node: &JsonValue, name: &str) -> Result<Value, Error> {
    field(node, name)?
        .as_i64()
        .map(Value::Integer)
        .ok_or_else(|| Error::Integer {
            field: name.to_string(),
        })
}

pub fn text(node: &JsonValue, name: &str) -> Result<Value, Error> {
    field(node, name)?
        .as_str()
        .map(|value| Value::Text(Arc::from(value)))
        .ok_or_else(|| Error::Text {
            field: name.to_string(),
        })
}

/// Decode a bounded literal payload, not a collection of child expressions.
pub fn list(node: &JsonValue, name: &str) -> Result<Value, Error> {
    let items = field(node, name)?.as_array().ok_or_else(|| Error::Array {
        field: name.to_string(),
    })?;
    if items.len() > 64 {
        return Err(Error::ListCapacity);
    }
    let values = items
        .iter()
        .enumerate()
        .map(|(index, item)| match item {
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
            JsonValue::Number(value) => value
                .as_i64()
                .map(Value::Integer)
                .ok_or(Error::ListElement { index }),
            JsonValue::String(value) => Ok(Value::Text(Arc::from(value.as_str()))),
            _ => Err(Error::ListElement { index }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    ClosedList::new(values).map(Value::List)
}

/// Decode flat literal entries, never child ASTs; preserve duplicates until checked.
pub fn record(node: &JsonValue, name: &str) -> Result<Value, Error> {
    let fields = field(node, name)?.as_array().ok_or_else(|| Error::Array {
        field: name.to_owned(),
    })?;
    record_capacity(fields.len())?;
    let mut values = BTreeMap::new();
    for (index, entry) in fields.iter().enumerate() {
        let name = field(entry, "name")?.as_str().ok_or_else(|| Error::Text {
            field: "name".to_owned(),
        })?;
        record_name(&values, name, index)?;
        // Do not even read the value until name and duplicate validation succeeds.
        let invalid = || Error::RecordElement {
            field: name.to_owned(),
        };
        let value = match field(entry, "value")? {
            JsonValue::Null => Value::Null,
            JsonValue::Bool(value) => Value::Boolean(*value),
            JsonValue::Number(value) => Value::Integer(value.as_i64().ok_or_else(invalid)?),
            JsonValue::String(value) => Value::Text(Arc::from(value.as_str())),
            _ => return Err(invalid()),
        };
        values.insert(name.to_owned(), value);
    }
    ClosedRecord::new(values.into_iter().collect()).map(Value::Record)
}

/// Legacy integer output admitted by generated CUE policy.
#[cfg(feature = "semantics")]
pub fn concrete(value: Value) -> Result<i64, Error> {
    encode_integer(generated_integer_output::integer_output(value)?)
}

/// Preserve exact JSON scalars and closed lists admitted by generated CUE policy.
#[cfg(feature = "semantics")]
pub fn concrete_json(value: Value) -> Result<JsonValue, Error> {
    encode_json(generated_json_output::json_output(value)?)
}

/// Extract an admitted integer; unsupported representations indicate a policy error.
pub fn encode_integer(value: Value) -> Result<i64, Error> {
    match value {
        Value::Integer(value) => Ok(value),
        _ => Err(Error::OutputPolicy),
    }
}

/// Encode admitted JSON scalars/closed lists without choosing incomplete diagnostics.
pub fn encode_json(value: Value) -> Result<JsonValue, Error> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Integer(value) => Ok(JsonValue::from(value)),
        Value::Boolean(value) => Ok(JsonValue::Bool(value)),
        Value::Text(value) => Ok(JsonValue::String(value.as_ref().to_owned())),
        Value::List(value) => value
            .as_slice()
            .iter()
            .cloned()
            .map(encode_json)
            .collect::<Result<Vec<_>, _>>()
            .map(JsonValue::Array),
        Value::Record(value) => value
            .as_map()
            .iter()
            .map(|(name, value)| Ok((name.clone(), encode_json(value.clone())?)))
            .collect::<Result<serde_json::Map<String, JsonValue>, Error>>()
            .map(JsonValue::Object),
        _ => Err(Error::OutputPolicy),
    }
}

/// Checked integer negation primitive; dispatch belongs to CUE control.
pub fn integer_negate(value: &Value) -> Result<Value, Error> {
    let Value::Integer(value) = value else {
        return Err(Error::Arithmetic);
    };
    value
        .checked_neg()
        .map(Value::Integer)
        .ok_or(Error::NegationOverflow)
}

/// Checked integer primitive; overload selection belongs to CUE control.
pub fn integer_sum(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_add(*right)
        .map(Value::Integer)
        .ok_or(Error::Overflow)
}

fn text_buffer(left_len: usize, right_len: usize) -> Result<String, Error> {
    let capacity = left_len.checked_add(right_len).ok_or(Error::TextCapacity)?;
    let mut buffer = String::new();
    buffer
        .try_reserve_exact(capacity)
        .map_err(|_| Error::TextCapacity)?;
    Ok(buffer)
}

/// Checked text primitive; concatenation preserves the original UTF-8 bytes.
pub fn text_concat(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Text(left), Value::Text(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    let mut buffer = text_buffer(left.len(), right.len())?;
    buffer.push_str(left);
    buffer.push_str(right);
    Ok(Value::Text(Arc::from(buffer)))
}

fn checked_text_length(length: usize) -> Result<Value, Error> {
    i64::try_from(length)
        .map(Value::Integer)
        .map_err(|_| Error::TextLengthOverflow)
}

/// Borrow the text's UTF-8 byte count without cloning or normalizing it.
pub fn text_length(value: &Value) -> Result<Value, Error> {
    let Value::Text(value) = value else {
        return Err(Error::Length);
    };
    checked_text_length(value.len())
}

/// Measure text using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn string_length(value: Value) -> Result<Value, Error> {
    generated_string_length::string_length(value)
}

/// Borrow a closed list's bounded count without cloning its contents.
pub fn list_count(value: &Value) -> Result<Value, Error> {
    let Value::List(value) = value else {
        return Err(Error::ListLength);
    };
    i64::try_from(value.len())
        .map(Value::Integer)
        .map_err(|_| Error::ListLength)
}

/// Defensively extract one scalar; CUE owns default domain/range admission.
pub fn list_at(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::List(list), Value::Integer(index)) = (left, right) else {
        return Err(Error::IndexOperands);
    };
    let index = usize::try_from(*index).map_err(|_| Error::IndexRange)?;
    list.as_slice().get(index).cloned().ok_or(Error::IndexRange)
}

/// Defensively copy a bounded slice of immutable child handles and recertify.
/// CUE owns default domain/range selection; this kernel also rejects misuse.
pub fn list_slice_elements(left: &Value, lower: &Value, upper: &Value) -> Result<Value, Error> {
    let (Value::List(list), Value::Integer(lower), Value::Integer(upper)) = (left, lower, upper)
    else {
        return Err(Error::ListSliceOperands);
    };
    let lower = usize::try_from(*lower).map_err(|_| Error::ListSliceRange)?;
    let upper = usize::try_from(*upper).map_err(|_| Error::ListSliceRange)?;
    let elements = list
        .as_slice()
        .get(lower..upper)
        .ok_or(Error::ListSliceRange)?;
    ClosedList::new(elements.to_vec()).map(Value::List)
}

/// Slice using generated CUE three-input domain/range policy.
#[cfg(feature = "semantics")]
pub fn list_slice(left: Value, lower: Value, upper: Value) -> Result<Value, Error> {
    generated_list_slice::list_slice(left, lower, upper)
}

/// Checked field presence; domain admission remains CUE-owned.
pub fn record_has(left: &Value, right: &Value) -> Result<bool, Error> {
    let (Value::Record(record), Value::Text(name)) = (left, right) else {
        return Err(Error::RecordIndexOperands);
    };
    Ok(record.as_map().contains_key(name.as_ref()))
}

/// Clone a selected scalar without retaining its parent record.
pub fn record_get(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Record(record), Value::Text(name)) = (left, right) else {
        return Err(Error::RecordIndexOperands);
    };
    record
        .as_map()
        .get(name.as_ref())
        .cloned()
        .ok_or_else(|| Error::RecordMissing {
            field: name.to_string(),
        })
}

/// Borrow the bounded field count, without inspecting field values.
pub fn record_count(value: &Value) -> Result<Value, Error> {
    let Value::Record(record) = value else {
        return Err(Error::RecordLength);
    };
    i64::try_from(record.len())
        .map(Value::Integer)
        .map_err(|_| Error::RecordLength)
}

#[cfg(feature = "semantics")]
pub fn record_index(left: Value, right: Value) -> Result<Value, Error> {
    generated_record_index::record_index(left, right)
}

#[cfg(feature = "semantics")]
pub fn record_length(value: Value) -> Result<Value, Error> {
    generated_record_length::record_length(value)
}

/// Index a closed collection using generated CUE domain and selection policy.
#[cfg(feature = "semantics")]
pub fn index(left: Value, right: Value) -> Result<Value, Error> {
    generated_index::index(left, right)
}

/// Index a closed list using generated CUE domain and range policy.
#[cfg(feature = "semantics")]
pub fn list_index(left: Value, right: Value) -> Result<Value, Error> {
    generated_list_index::list_index(left, right)
}

/// Select text-byte or list-element counting using CUE policy.
#[cfg(feature = "semantics")]
pub fn length(value: Value) -> Result<Value, Error> {
    generated_length::length(value)
}

/// Measure a list using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn list_length(value: Value) -> Result<Value, Error> {
    generated_list_length::list_length(value)
}

/// Add using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn sum(left: Value, right: Value) -> Result<Value, Error> {
    generated_sum::sum(left, right)
}

/// Compare using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_equal::equal(left, right)
}

/// Compare using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn not_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_not_equal::not_equal(left, right)
}

/// Order using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn less(left: Value, right: Value) -> Result<Value, Error> {
    generated_less::less(left, right)
}

#[cfg(feature = "semantics")]
pub fn less_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_less_equal::less_equal(left, right)
}

#[cfg(feature = "semantics")]
pub fn greater(left: Value, right: Value) -> Result<Value, Error> {
    generated_greater::greater(left, right)
}

#[cfg(feature = "semantics")]
pub fn greater_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_greater_equal::greater_equal(left, right)
}

/// Select a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_and(left: Value, right: Value) -> Result<Value, Error> {
    generated_logical_and::logical_and(left, right)
}

/// Select a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_or(left: Value, right: Value) -> Result<Value, Error> {
    generated_logical_or::logical_or(left, right)
}

/// Invert a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_not(value: Value) -> Result<Value, Error> {
    generated_logical_not::logical_not(value)
}

/// Negate using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn negate(value: Value) -> Result<Value, Error> {
    generated_negate::negate(value)
}

/// Subtract using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn difference(left: Value, right: Value) -> Result<Value, Error> {
    generated_difference::difference(left, right)
}

/// Multiply using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn product(left: Value, right: Value) -> Result<Value, Error> {
    generated_product::product(left, right)
}

/// Checked integer subtraction primitive; dispatch belongs to CUE control.
pub fn integer_difference(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_sub(*right)
        .map(Value::Integer)
        .ok_or(Error::DifferenceOverflow)
}

/// Checked integer multiplication primitive; dispatch belongs to CUE control.
pub fn integer_product(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_mul(*right)
        .map(Value::Integer)
        .ok_or(Error::ProductOverflow)
}

/// Divide using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn divide(left: Value, right: Value) -> Result<Value, Error> {
    generated_divide::divide(left, right)
}

/// Compute Euclidean remainder using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn modulo(left: Value, right: Value) -> Result<Value, Error> {
    generated_modulo::modulo(left, right)
}

/// Compute truncating quotient using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn quotient(left: Value, right: Value) -> Result<Value, Error> {
    generated_quotient::quotient(left, right)
}

/// Compute truncating remainder using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn remainder(left: Value, right: Value) -> Result<Value, Error> {
    generated_remainder::remainder(left, right)
}

/// Checked Euclidean division primitive; dispatch belongs to CUE control.
pub fn integer_divide(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_div_euclid(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked Euclidean remainder primitive; unit-divisor policy belongs to CUE.
pub fn integer_modulo(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_rem_euclid(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked truncating division primitive; dispatch belongs to CUE control.
pub fn integer_quotient(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_div(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked truncating remainder primitive; unit-divisor policy belongs to CUE.
pub fn integer_remainder(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_rem(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Construct an inclusive lower bound using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn lower_bound(value: Value) -> Result<Value, Error> {
    generated_lower_bound::lower_bound(value)
}

/// Construct an inclusive upper bound using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn upper_bound(value: Value) -> Result<Value, Error> {
    generated_upper_bound::upper_bound(value)
}

/// Construct an exclusive lower bound using CUE domain/extrema/offset policy.
#[cfg(feature = "semantics")]
pub fn lower_exclusive_bound(value: Value) -> Result<Value, Error> {
    generated_lower_exclusive_bound::lower_exclusive_bound(value)
}

/// Construct an exclusive upper bound using CUE domain/extrema/offset policy.
#[cfg(feature = "semantics")]
pub fn upper_exclusive_bound(value: Value) -> Result<Value, Error> {
    generated_upper_exclusive_bound::upper_exclusive_bound(value)
}

/// Unify using the generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn unify(left: Value, right: Value) -> Result<Value, Error> {
    generated_unification::unify(left, right)
}

#[cfg(test)]
mod tests {
    use super::{
        EvaluationContext, Value, checked_text_length, integer_sum, text_buffer, text_concat,
    };
    use crate::Error;
    use std::sync::Arc;

    fn nested_visits(context: &mut EvaluationContext, levels: usize) -> Result<(), Error> {
        context.visit(|context| {
            if levels == 1 {
                Ok(())
            } else {
                nested_visits(context, levels - 1)
            }
        })
    }

    #[test]
    fn evaluation_depth_boundary_and_restoration() {
        let mut context = EvaluationContext::default();
        assert!(nested_visits(&mut context, 64).is_ok());
        assert_eq!((context.remaining, context.depth), (4096 - 64, 0));
        assert!(matches!(
            nested_visits(&mut context, 65),
            Err(Error::EvaluationDepth)
        ));
        // Only the 64 accepted entries were charged; the error unwound them all.
        assert_eq!((context.remaining, context.depth), (4096 - 128, 0));
        assert!(nested_visits(&mut context, 64).is_ok());
        assert_eq!((context.remaining, context.depth), (4096 - 192, 0));
    }

    #[test]
    fn evaluation_errors_consume_visits_without_replenishing() {
        let mut context = EvaluationContext::default();
        for remaining in (0..4096).rev() {
            let result: Result<(), Error> = context.visit(|context| {
                assert_eq!(context.depth, 1);
                Err(Error::Kind)
            });
            assert!(matches!(result, Err(Error::Kind)));
            assert_eq!((context.remaining, context.depth), (remaining, 0));
        }
        assert!(matches!(
            context.visit::<()>(|_| panic!("exhausted callback must not run")),
            Err(Error::EvaluationVisits)
        ));
        assert_eq!((context.remaining, context.depth), (0, 0));
        assert!(EvaluationContext::default().visit(|_| Ok(())).is_ok());
    }

    #[test]
    fn evaluation_visit_priority_at_depth_boundary() {
        let mut context = EvaluationContext::default();
        for _ in 0..(4096 - 64) {
            assert!(context.visit(|_| Ok(())).is_ok());
        }
        // The final permitted entry is also at the depth limit. Both guards
        // reject its child, so visit exhaustion must win over active depth.
        assert!(matches!(
            nested_visits(&mut context, 65),
            Err(Error::EvaluationVisits)
        ));
        assert_eq!((context.remaining, context.depth), (0, 0));
        assert_eq!(
            Error::EvaluationVisits.to_string(),
            "evaluation exceeds 4096 AST visits"
        );
        assert_eq!(
            Error::EvaluationDepth.to_string(),
            "evaluation exceeds 64 active AST nodes"
        );
    }

    #[test]
    fn text_length_conversion_extrema_without_allocation() {
        assert_eq!(checked_text_length(0).ok(), Some(Value::Integer(0)));
        assert_eq!(checked_text_length(1).ok(), Some(Value::Integer(1)));
        if let Ok(maximum) = usize::try_from(i64::MAX) {
            assert_eq!(
                checked_text_length(maximum).ok(),
                Some(Value::Integer(i64::MAX))
            );
            if let Some(overflow) = maximum.checked_add(1) {
                assert!(matches!(
                    checked_text_length(overflow),
                    Err(Error::TextLengthOverflow)
                ));
            }
        }
        match i64::try_from(usize::MAX) {
            Ok(maximum) => assert_eq!(
                checked_text_length(usize::MAX).ok(),
                Some(Value::Integer(maximum))
            ),
            Err(_) => assert!(matches!(
                checked_text_length(usize::MAX),
                Err(Error::TextLengthOverflow)
            )),
        }
    }

    #[test]
    #[cfg(feature = "semantics")]
    fn null_and_top_output() {
        assert!(matches!(
            super::concrete(Value::Null),
            Err(Error::IntegerOutput)
        ));
        assert!(matches!(
            super::concrete(Value::Top),
            Err(Error::IncompleteValue)
        ));
        assert_eq!(
            super::concrete_json(Value::Null).ok(),
            Some(serde_json::Value::Null)
        );
        assert!(matches!(
            super::concrete_json(Value::Top),
            Err(Error::IncompleteValue)
        ));
    }

    #[test]
    fn checked_output_encoders() {
        for value in [i64::MIN, 0, i64::MAX] {
            assert_eq!(
                super::encode_integer(Value::Integer(value)).ok(),
                Some(value)
            );
            assert_eq!(
                super::encode_json(Value::Integer(value)).ok(),
                Some(serde_json::Value::from(value))
            );
        }
        assert_eq!(
            super::encode_json(Value::Null).ok(),
            Some(serde_json::Value::Null)
        );
        for value in [false, true] {
            assert_eq!(
                super::encode_json(Value::Boolean(value)).ok(),
                Some(serde_json::Value::Bool(value))
            );
        }
        let text: Arc<str> = Arc::from("é\n\"\\");
        assert_eq!(
            super::encode_json(Value::Text(Arc::clone(&text))).ok(),
            Some(serde_json::Value::String("é\n\"\\".to_owned()))
        );
        assert_eq!(Arc::strong_count(&text), 1);
        for value in [
            Value::Top,
            Value::Int,
            Value::Bool,
            Value::String,
            Value::Bounds {
                lower: Some(0),
                upper: Some(0),
            },
        ] {
            assert!(matches!(
                super::encode_json(value.clone()),
                Err(Error::OutputPolicy)
            ));
            assert!(matches!(
                super::encode_integer(value),
                Err(Error::OutputPolicy)
            ));
        }
        for value in [Value::Null, Value::Boolean(false), Value::Text(text)] {
            assert!(matches!(
                super::encode_integer(value),
                Err(Error::OutputPolicy)
            ));
        }
        assert_eq!(
            Error::OutputPolicy.to_string(),
            "output policy returned an unsupported value"
        );
    }

    #[test]
    fn checked_integer_kernels() {
        use super::{integer_difference, integer_negate, integer_product};

        assert_eq!(
            integer_difference(&Value::Integer(i64::MIN), &Value::Integer(i64::MIN)).ok(),
            Some(Value::Integer(0))
        );
        assert!(matches!(
            integer_negate(&Value::Integer(i64::MIN)),
            Err(Error::NegationOverflow)
        ));
        assert!(matches!(
            integer_difference(&Value::Integer(i64::MAX), &Value::Integer(-1)),
            Err(Error::DifferenceOverflow)
        ));
        assert!(matches!(
            integer_product(&Value::Integer(i64::MIN), &Value::Integer(-1)),
            Err(Error::ProductOverflow)
        ));
        for value in [
            Value::Null,
            Value::Top,
            Value::Int,
            Value::Bool,
            Value::Bounds {
                lower: None,
                upper: None,
            },
            Value::Text(Arc::from("x")),
        ] {
            assert!(matches!(integer_negate(&value), Err(Error::Arithmetic)));
            assert!(matches!(
                integer_difference(&value, &Value::Integer(1)),
                Err(Error::Arithmetic)
            ));
            assert!(matches!(
                integer_product(&Value::Integer(1), &value),
                Err(Error::Arithmetic)
            ));
        }
    }

    #[test]
    fn checked_division_kernels() {
        use super::{integer_divide, integer_modulo, integer_quotient, integer_remainder};

        for (kernel, expected) in [
            (
                integer_divide as fn(&Value, &Value) -> Result<Value, Error>,
                -3,
            ),
            (integer_modulo, 2),
            (integer_quotient, -2),
            (integer_remainder, -1),
        ] {
            assert_eq!(
                kernel(&Value::Integer(-7), &Value::Integer(3)).ok(),
                Some(Value::Integer(expected))
            );
            assert!(matches!(
                kernel(&Value::Integer(i64::MIN), &Value::Integer(-1)),
                Err(Error::DivisionOverflow)
            ));
            assert!(matches!(
                kernel(&Value::Integer(1), &Value::Integer(0)),
                Err(Error::DivisionByZero)
            ));
            for value in [
                Value::Null,
                Value::Top,
                Value::Int,
                Value::Bool,
                Value::Boolean(false),
                Value::String,
                Value::Text(Arc::from("x")),
                Value::Bounds {
                    lower: Some(0),
                    upper: Some(0),
                },
            ] {
                assert!(matches!(
                    kernel(&value, &Value::Integer(0)),
                    Err(Error::Arithmetic)
                ));
                assert!(matches!(
                    kernel(&Value::Integer(1), &value),
                    Err(Error::Arithmetic)
                ));
            }
        }
    }

    #[test]
    fn checked_sum_and_concat_kernels() {
        assert_eq!(
            integer_sum(&Value::Integer(20), &Value::Integer(22)).ok(),
            Some(Value::Integer(42))
        );
        assert!(matches!(
            integer_sum(&Value::Integer(i64::MAX), &Value::Integer(1)),
            Err(Error::Overflow)
        ));
        let left: Arc<str> = Arc::from("é");
        let value = Value::Text(Arc::clone(&left));
        let result = text_concat(&value, &Value::Text(Arc::from("\u{301}")));
        assert_eq!(result.ok(), Some(Value::Text(Arc::from("é\u{301}"))));
        assert_eq!(&*left, "é");
        assert!(matches!(
            text_concat(&value, &Value::Int),
            Err(Error::Arithmetic)
        ));
        assert!(matches!(
            integer_sum(&value, &Value::Int),
            Err(Error::Arithmetic)
        ));
        assert!(matches!(
            text_buffer(usize::MAX, 1),
            Err(Error::TextCapacity)
        ));
        assert!(matches!(
            text_buffer(usize::MAX, 0),
            Err(Error::TextCapacity)
        ));
    }
}
