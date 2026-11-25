//! Explicit JSON DATA: at most 1 MiB UTF-8 bytes and 64 container levels.
//! Scalars have depth zero; an object/array root has depth one. No CUE fallback.
//! Size rejection precedes UTF-8 validation. Within that bound, UTF-8 is checked
//! in full before grammar traversal; opening container 65 stops traversal before
//! its contents. Earlier syntax errors and trailing documents remain invalid.
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::{cell::Cell, fmt, io::Read};

const MAX_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 64;

pub(super) fn read(reader: impl Read) -> Result<Value, String> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read JSON input: {error}"))?;
    if bytes.len() > MAX_BYTES {
        return Err("unsupported JSON: input exceeds 1 MiB".into());
    }
    std::str::from_utf8(&bytes).map_err(|error| format!("invalid JSON: {error}"))?;
    let excluded = Cell::new(false);
    prepare(&mut bytes, &excluded);
    let too_deep = Cell::new(false);
    let mut decoder = serde_json::Deserializer::from_slice(&bytes);
    let value = Data {
        excluded: &excluded,
        too_deep: &too_deep,
        depth: 0,
    }
    .deserialize(&mut decoder)
    .map_err(|error| {
        let category = if too_deep.get() {
            "unsupported JSON"
        } else {
            "invalid JSON"
        };
        format!("{category}: {error}")
    })?;
    decoder
        .end()
        .map_err(|error| format!("invalid JSON: {error}"))?;
    if excluded.get() {
        return Err(
            "unsupported JSON: requires unique decoded keys and decimal i64 integers (no -0)"
                .into(),
        );
    }
    Ok(value)
}

// serde_json normally parses some oversized integers through f64, and reports
// huge exponents as syntax errors. Replace ONLY syntactically valid excluded
// number tokens with zero before decoding, remembering rejection. Malformed
// syntax, Unicode, and trailing documents are still validated by serde. Never
// return the substituted value. Strings (including escaped quotes) are opaque.
fn prepare(bytes: &mut [u8], excluded: &Cell<bool>) {
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < bytes.len()
                    && matches!(bytes[i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
                {
                    i += 1;
                }
                let token = &bytes[start..i];
                if number(token)
                    && (token == b"-0"
                        || !std::str::from_utf8(token)
                            .is_ok_and(|text| text.parse::<i64>().is_ok()))
                {
                    excluded.set(true);
                    bytes[start..i].fill(b' ');
                    bytes[start] = b'0';
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
}

// JSON number grammar, not a floating-point conversion (no rounding/overflow).
fn number(mut token: &[u8]) -> bool {
    if token.first() == Some(&b'-') {
        token = &token[1..];
    }
    match token.first() {
        Some(b'0') => token = &token[1..],
        Some(b'1'..=b'9') => {
            token = &token[1..];
            while token.first().is_some_and(u8::is_ascii_digit) {
                token = &token[1..];
            }
        }
        _ => return false,
    }
    if token.first() == Some(&b'.') {
        token = &token[1..];
        let before = token.len();
        while token.first().is_some_and(u8::is_ascii_digit) {
            token = &token[1..];
        }
        if token.len() == before {
            return false;
        }
    }
    if matches!(token.first(), Some(b'e' | b'E')) {
        token = &token[1..];
        if matches!(token.first(), Some(b'+' | b'-')) {
            token = &token[1..];
        }
        let before = token.len();
        while token.first().is_some_and(u8::is_ascii_digit) {
            token = &token[1..];
        }
        if token.len() == before {
            return false;
        }
    }
    token.is_empty()
}

#[derive(Clone, Copy)]
struct Data<'a> {
    excluded: &'a Cell<bool>,
    too_deep: &'a Cell<bool>,
    depth: usize,
}

impl Data<'_> {
    // Check only containers reached by the first document's grammar. Stop at
    // level 65 before inspecting its contents; later syntax is not validated.
    fn container<E: serde::de::Error>(self) -> Result<Self, E> {
        if self.depth == MAX_DEPTH {
            self.too_deep.set(true);
            return Err(E::custom("container nesting exceeds 64 levels"));
        }
        Ok(Self {
            depth: self.depth + 1,
            ..self
        })
    }
}

impl<'de> DeserializeSeed<'de> for Data<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        decoder.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Data<'_> {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON DATA value")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> {
        match i64::try_from(value) {
            Ok(value) => Ok(value.into()),
            Err(_) => {
                self.excluded.set(true);
                Ok(Value::Null)
            }
        }
    }
    fn visit_f64<E: serde::de::Error>(self, _value: f64) -> Result<Value, E> {
        self.excluded.set(true);
        Ok(Value::Null)
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let child = self.container::<A::Error>()?;
        let mut fields = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            let value = access.next_value_seed(child)?;
            match fields.entry(key) {
                serde_json::map::Entry::Occupied(_) => self.excluded.set(true),
                serde_json::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
        Ok(Value::Object(fields))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let child = self.container::<A::Error>()?;
        let mut items = Vec::new();
        while let Some(value) = access.next_element_seed(child)? {
            items.push(value);
        }
        Ok(Value::Array(items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(text: &[u8], prefix: &str) {
        let error = read(text).expect_err("profile or syntax rejection");
        assert!(error.starts_with(prefix), "{error}");
    }

    #[test]
    fn exact_data() {
        let input = br#"{"a":[null,true,false,-9223372036854775808,9223372036854775807],"s":"\uD83D\uDE00","":{}}"#;
        assert_eq!(
            read(&input[..]).expect("exact supported data"),
            serde_json::json!({
                "a": [null, true, false, i64::MIN, i64::MAX], "s": "😀", "": {}
            })
        );
        assert_eq!(
            read(&b" \n0\t"[..]).expect("whitespace around zero"),
            Value::from(0)
        );
        assert_eq!(
            read(&br#""-0 1e999 \" { [""#[..]).expect("opaque string content"),
            Value::from("-0 1e999 \" { [")
        );
    }

    #[test]
    fn exclusions_and_malformed_are_distinct() {
        let numbers =
            "-0 1.0 1e0 1E+999999999 -1e-999999999 9223372036854775808 -9223372036854775809";
        for input in numbers.split_whitespace().chain([
            r#"{"a":1,"a":1}"#,
            r#"{"a":1,"\u0061":2}"#,
            r#"[{"😀":0,"\ud83d\ude00":0}]"#,
        ]) {
            rejected(input.as_bytes(), "unsupported JSON:");
        }
        let malformed =
            "|01|-01|+1|1.|1e|1e+|NaN|Infinity|{} []|0 1|{|[0,]|// CUE\n0|40 + 2|value: 1";
        for input in malformed.split('|').chain([
            r#""\ud800""#,
            r#""\udc00""#,
            r#""\ud800\u1234""#,
            r#""\U00000041""#,
            r#"{"a":1,"a":2,}"#,
            "1e999999999 x",
            "1e999999999 2",
        ]) {
            rejected(input.as_bytes(), "invalid JSON:");
        }
        rejected(b"\xff", "invalid JSON:");
        rejected(b"\"\xff\"", "invalid JSON:");
        rejected("\u{feff}0".as_bytes(), "invalid JSON:");
    }

    #[test]
    fn depth_checks_follow_first_document_grammar() {
        // Neither an invalid prefix nor a second document may reach the guard.
        for (open, close) in [("[", "]"), (r#"{"a":"#, "}")] {
            for depth in [65, 2000] {
                let deep = format!("{}0{}", open.repeat(depth), close.repeat(depth));
                for prefix in ["x", "0 ", "{} ", "null ", "-0 ", r#"{"a":0 "b":"#, "[0 "] {
                    rejected(format!("{prefix}{deep}").as_bytes(), "invalid JSON:");
                }
                let bad_nested = format!("{}x{deep}", "[".repeat(64));
                rejected(bad_nested.as_bytes(), "invalid JSON:");
                rejected(deep.as_bytes(), "unsupported JSON:");
            }
        }
        rejected(format!("x{}", "[".repeat(65)).as_bytes(), "invalid JSON:");
        // A valid first-document prefix reaching level 65 stops immediately;
        // grammar beyond that opening is intentionally not inspected.
        rejected(
            format!("{}!", "[".repeat(65)).as_bytes(),
            "unsupported JSON:",
        );
    }

    #[test]
    fn exact_limits_and_bounded_read() {
        let mut bytes = vec![b' '; MAX_BYTES];
        bytes[0] = b'0';
        assert_eq!(read(&bytes[..]).expect("exact byte limit"), Value::from(0));
        // Multi-byte UTF-8 counts in bytes, not characters.
        let text = format!("\"{}\"", "é".repeat((MAX_BYTES - 2) / 2));
        assert!(read(text.as_bytes()).is_ok());
        rejected(format!("{text} ").as_bytes(), "unsupported JSON:");
        let mut remaining = std::io::Cursor::new(vec![b' '; MAX_BYTES + 100]);
        assert!(
            read(&mut remaining)
                .expect_err("bounded read overflow")
                .starts_with("unsupported JSON:")
        );
        assert_eq!(remaining.position(), (MAX_BYTES + 1) as u64);
        for depth in [64, 65] {
            for (open, close) in [("[", "]"), (r#"{"a":"#, "}")] {
                let text = format!("{}0{}", open.repeat(depth), close.repeat(depth));
                if depth == 64 {
                    assert!(read(text.as_bytes()).is_ok());
                } else {
                    rejected(text.as_bytes(), "unsupported JSON:");
                }
            }
        }
    }
}
