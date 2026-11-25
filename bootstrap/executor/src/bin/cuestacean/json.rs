//! Explicit JSON DATA: at most 1 MiB UTF-8 bytes and 64 container levels.
//! Scalars have depth zero; an object/array root has depth one. No CUE fallback.
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
    prepare(&mut bytes, &excluded)?;
    let mut decoder = serde_json::Deserializer::from_slice(&bytes);
    let value = Data(&excluded)
        .deserialize(&mut decoder)
        .map_err(|error| format!("invalid JSON: {error}"))?;
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
fn prepare(bytes: &mut [u8], excluded: &Cell<bool>) -> Result<(), String> {
    let (mut i, mut depth) = (0, 0usize);
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
            b'[' | b'{' => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err("unsupported JSON: container nesting exceeds 64 levels".into());
                }
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
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
                        || std::str::from_utf8(token).unwrap().parse::<i64>().is_err())
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
    Ok(())
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
struct Data<'a>(&'a Cell<bool>);

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
                self.0.set(true);
                Ok(Value::Null)
            }
        }
    }
    fn visit_f64<E: serde::de::Error>(self, _value: f64) -> Result<Value, E> {
        self.0.set(true);
        Ok(Value::Null)
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut fields = Map::new();
        while let Some(key) = access.next_key::<String>()? {
            let value = access.next_value_seed(self)?;
            match fields.entry(key) {
                serde_json::map::Entry::Occupied(_) => self.0.set(true),
                serde_json::map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
        Ok(Value::Object(fields))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(value) = access.next_element_seed(self)? {
            items.push(value);
        }
        Ok(Value::Array(items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(text: &[u8], prefix: &str) {
        let error = read(text).unwrap_err();
        assert!(error.starts_with(prefix), "{error}");
    }

    #[test]
    fn exact_data() {
        let input = br#"{"a":[null,true,false,-9223372036854775808,9223372036854775807],"s":"\uD83D\uDE00","":{}}"#;
        assert_eq!(
            read(&input[..]).unwrap(),
            serde_json::json!({
                "a": [null, true, false, i64::MIN, i64::MAX], "s": "😀", "": {}
            })
        );
        assert_eq!(read(&b" \n0\t"[..]).unwrap(), Value::from(0));
        assert_eq!(
            read(&br#""-0 1e999 \" { [""#[..]).unwrap(),
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
    fn exact_limits_and_bounded_read() {
        let mut bytes = vec![b' '; MAX_BYTES];
        bytes[0] = b'0';
        assert_eq!(read(&bytes[..]).unwrap(), Value::from(0));
        // Multi-byte UTF-8 counts in bytes, not characters.
        let text = format!("\"{}\"", "é".repeat((MAX_BYTES - 2) / 2));
        assert!(read(text.as_bytes()).is_ok());
        rejected(format!("{text} ").as_bytes(), "unsupported JSON:");
        let mut remaining = std::io::Cursor::new(vec![b' '; MAX_BYTES + 100]);
        assert!(
            read(&mut remaining)
                .unwrap_err()
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
