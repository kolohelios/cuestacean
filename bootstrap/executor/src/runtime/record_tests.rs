use super::*;
use serde_json::json;

macro_rules! rejects {
    ($expression:expr, $pattern:pat) => {
        assert!(matches!($expression, Err($pattern)))
    };
}

#[test]
fn constructor_and_codec_order() {
    let entry = |name: &str, value| (name.to_owned(), value);
    rejects!(
        ClosedRecord::new(vec![entry("", Value::Top); 65]),
        Error::RecordCapacity
    );
    rejects!(
        ClosedRecord::new(vec![entry("", Value::Top)]),
        Error::RecordName { index: 0 }
    );
    let duplicate = vec![entry("a", Value::Null), entry("a", Value::Top)];
    rejects!(ClosedRecord::new(duplicate), Error::RecordDuplicate { .. });
    for value in [Value::Top, Value::Int] {
        rejects!(
            ClosedRecord::new(vec![entry("a", value)]),
            Error::RecordElement { .. }
        );
    }
    let decode = |fields| record(&json!({"fields": fields}), "fields");
    rejects!(decode(json!(vec![json!({}); 65])), Error::RecordCapacity);
    rejects!(decode(json!([{"name":""}])), Error::RecordName { .. });
    let duplicate = json!([{"name":"a","value":null},{"name":"a"}]);
    rejects!(decode(duplicate), Error::RecordDuplicate { .. });
    for value in [
        json!([]),
        json!({}),
        json!(1.5),
        json!(9223372036854775808u64),
    ] {
        rejects!(
            decode(json!([{"name":"a","value":value}])),
            Error::RecordElement { .. }
        );
    }
}

#[test]
fn shared_ownership_and_defensive_domains() {
    let text: Arc<str> = Arc::from("é");
    let record =
        ClosedRecord::new(vec![("a".into(), Value::Text(Arc::clone(&text)))]).expect("record");
    let shared = record.clone();
    assert!(Arc::ptr_eq(&record.values, &shared.values));
    let value = Value::Record(record);
    let key = Value::Text(Arc::from("a"));
    assert_eq!(record_count(&value).expect("count"), Value::Integer(1));
    assert!(record_has(&value, &key).expect("present"));
    let missing = Value::Text(Arc::from("missing"));
    assert!(!record_has(&value, &missing).expect("absent"));
    rejects!(record_get(&value, &missing), Error::RecordMissing { .. });
    let selected = record_get(&value, &key).expect("lookup");
    rejects!(
        ClosedList::new(vec![Value::String]),
        Error::ListElement { index: 0 }
    );
    drop(value);
    drop(shared);
    assert_eq!(selected, Value::Text(Arc::clone(&text)));
    assert_eq!(Arc::strong_count(&text), 2);
    rejects!(record_has(&Value::Null, &key), Error::RecordIndexOperands);
    rejects!(
        record_get(&selected, &Value::Null),
        Error::RecordIndexOperands
    );
    rejects!(record_count(&Value::Null), Error::RecordLength);
}
