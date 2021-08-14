use serde_json::{Result, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum JsonV<'a> {
    Null(Option<Status<'a>>),
    String(String, Option<Status<'a>>),
    Bool(bool, Option<Status<'a>>),
    Number(f64, Option<Status<'a>>),
    Array(Vec<JsonV<'a>>, Option<Status<'a>>),
    Object(HashMap<String, JsonV<'a>>, Option<Status<'a>>),
}

type JsonVPair<'a> = (JsonV<'a>, JsonV<'a>);

#[derive(Clone, Debug)]
enum DiffType<'a> {
    ObjectKeyMissing(&'a str, JsonV<'a>),
    ObjectKeyPresent(&'a str, JsonV<'a>),
    ObjectValueDiff(&'a str, JsonVPair<'a>),
    Element(JsonV<'a>, JsonVPair<'a>),
    Primitive(JsonVPair<'a>),
}

#[derive(Clone, Debug)]
struct Status<'a> {
    different_values: Vec<DiffType<'a>>,
}

struct DiffResult {
    value: Value,
    is_diff: bool,
}

fn main() {
    println!("Hello, world!");

    diff("", "");
}

fn diff(a: &str, b: &str) -> Result<()> {
    let ja: Value = serde_json::from_str(a)?;
    let jb: Value = serde_json::from_str(b)?;

    diff_rec(&ja, &jb);

    Ok(())
}

fn diff_rec<'a>(a: &'a Value, b: &'a Value) -> JsonV<'a> {
    match (a, b) {
        // Check keys first then values
        (Value::Object(p1), Value::Object(p2)) => {
            // Find fields not in the other and vice versa
            let a: Vec<_> = p1.keys().collect();
            let b: Vec<_> = p2.keys().collect();
            let fields_in_b_not_a = a_intersection_complement_b(b.clone(), a.clone());
            let fields_in_a_not_b = a_intersection_complement_b(a.clone(), b.clone());

            // Find fields where the values differ in the two structures

            let mut value_diff: Vec<DiffType> = Vec::new();
            for k in union(a.clone(), b.clone()) {
                if let (Some(v1), Some(v2)) = (p1.get(k), p2.get(k)) {
                    let res = diff_rec(v1, v2);
                    println!("1. {} {:?}", k, res);
                    let r2 = get_status(res.clone());
                    if r2.is_some() {
                        value_diff.push(DiffType::ObjectValueDiff(k, (convert(v1), convert(v2))));
                    }
                }
            }

            let df = value_diff.clone();

            let res = if fields_in_a_not_b.is_empty()
                && fields_in_b_not_a.is_empty()
                && value_diff.is_empty()
            {
                // Objects are the same
                let f1 = |x: String| (x.to_string(), convert(p1.get(&x.to_string()).unwrap()));
                let hm: HashMap<String, JsonV> = a.iter().map(|y| f1(y.to_string())).collect();
                JsonV::Object(hm, None)
            } else {
                // Not the same
                let f1 = |x| DiffType::ObjectKeyPresent(x, convert(p1.get(x).unwrap()));
                let f2 = |x| DiffType::ObjectKeyMissing(x, convert(p2.get(x).unwrap()));
                let mut v1: Vec<DiffType> = fields_in_a_not_b.iter().map(|y| f1(y)).collect();
                let v2: Vec<DiffType> = fields_in_b_not_a.iter().map(|y| f2(y)).collect();

                v1.extend(v2);
                v1.extend(value_diff);

                JsonV::Object(
                    HashMap::new(),
                    Some(Status {
                        different_values: v1,
                    }),
                )
            };

            println!("{:?}", fields_in_a_not_b);
            println!("{:?}", fields_in_b_not_a);
            println!("{:?}", df);
            res
        }
        // Check equal number of elements and element equality
        (Value::Array(p1), Value::Array(p2)) => JsonV::Null(None),
        (Value::String(p1), Value::String(p2)) => {
            if p1 == p2 {
                JsonV::String(p1.to_string(), None)
            } else {
                let v1 = DiffType::Primitive((convert(a), convert(b)));
                JsonV::String(
                    "".to_string(),
                    Some(Status {
                        different_values: vec![v1],
                    }),
                )
            }
        }
        (Value::Number(p1), Value::Number(p2)) => {
            if cmp_option(p1.as_f64(), p2.as_f64()) {
                JsonV::Number(p1.as_f64().unwrap(), None)
            } else {
                let v1 = DiffType::Primitive((convert(a), convert(b)));
                JsonV::Number(
                    0.0,
                    Some(Status {
                        different_values: vec![v1],
                    }),
                )
            }
        }
        (Value::Bool(p1), Value::Bool(p2)) => {
            if p1 == p2 {
                JsonV::Bool(*p1, None)
            } else {
                let v1 = DiffType::Primitive((JsonV::Bool(*p1, None), JsonV::Bool(*p2, None)));
                JsonV::Bool(
                    false,
                    Some(Status {
                        different_values: vec![v1],
                    }),
                )
            }
        }
        (Value::Null, Value::Null) => JsonV::Null(None),
        _ => JsonV::Null(Some(Status {
            different_values: vec![],
        })),
    }
}

fn cmp_option<T: std::string::ToString>(a: Option<T>, b: Option<T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.to_string() == b.to_string(),
        (None, None) => false,
        _ => false,
    }
}

fn convert<'a>(a: &Value) -> JsonV<'a> {
    match a {
        Value::Null => JsonV::Null(None),
        Value::Bool(b) => JsonV::Bool(*b, None),
        Value::Number(b) => {
            if let Some(n) = b.as_f64() {
                JsonV::Number(n, None)
            } else {
                JsonV::Number(0.0, None)
            }
        }
        Value::String(b) => JsonV::String(b.to_string(), None),
        Value::Object(b) => {
            let mut map: HashMap<String, JsonV> = HashMap::new();
            for k in b.keys() {
                if let Some(v) = b.get(k) {
                    map.insert(k.to_string(), convert(v));
                }
            }
            JsonV::Object(map, None)
        }
        Value::Array(b) => {
            let na = b.iter().map(convert).collect();
            JsonV::Array(na, None)
        }
    }
}

fn get_status(j: JsonV) -> Option<Status> {
    match j {
        JsonV::Null(st) => st,
        JsonV::String(_, st) => st,
        JsonV::Bool(_, st) => st,
        JsonV::Number(_, st) => st,
        JsonV::Object(_, st) => st,
        JsonV::Array(_, st) => st,
    }
}

fn a_intersection_complement_b<T: std::clone::Clone + std::cmp::Ord>(
    a: Vec<T>,
    b: Vec<T>,
) -> Vec<T> {
    let u = union(a, b.clone());
    let mut u2 = u.clone();
    u2.retain(|x| !b.contains(x));

    u2
}

fn union<T: std::clone::Clone + std::cmp::Ord>(s: Vec<T>, other: Vec<T>) -> Vec<T> {
    let mut stack = s.clone();
    for x in other {
        stack.push(x)
    }
    stack.sort();
    stack.dedup();

    stack
}

fn eq_value(a: Value, b: Value) -> bool {
    println!("{}", json_type(&a));
    println!("{}", json_type(&b));
    return json_type(&a) == json_type(&b);
}

fn json_type_2<'a>(a: (&Value, &Value)) -> i32 {
    return match a {
        (Value::Object(p1), Value::Object(p2)) => 1,
        (Value::Array(p1), Value::Array(p2)) => 1,
        (Value::String(p1), Value::String(p2)) => 1,
        (Value::Number(p1), Value::Number(p2)) => 1,
        (Value::Bool(p1), Value::Bool(p2)) => 1,
        (Value::Null, Value::Null) => 6,
        _ => 7,
    };
}

fn json_type(a: &Value) -> i32 {
    return match a {
        Value::Object(_) => 1,
        Value::Array(_) => 2,
        Value::String(_) => 3,
        Value::Number(_) => 4,
        Value::Bool(_) => 5,
        Value::Null => 6,
    };
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_type() {
        let t = Value::String("test".to_string());
        let v = vec![Value::String("test2".to_string())];
        let s = Value::Array(v);

        assert_eq!(false, eq_value(t, s));
    }

    #[test]
    fn test_diff() {
        let r = r#"
            {
            "f1": "v2",
            "f2": {
                "f3": 123,
                "f4": 456
            }
        }"#;
        let r2 = r#"
            {
            "f1": "v2",
            "f2": {
                "f3": 456,
                "f4": 456
            }
        }"#;

        println!("{}", r2)
    }

    #[test]
    fn test_diff_2() -> Result<()> {
        let r = r#"
            {
            "f1": "v2",
            "f2": "v4",
            "f3": "v5"
        }"#;
        let r2 = r#"
            {
            "f1": "v2",
            "f2": {
                "f3": 456,
                "f4": 456
            },
            "f5": "v6"
        }"#;

        println!("{:?}", r);
        println!("{:?}", r2);

        // diff(r, r2);
        let ja: Value = serde_json::from_str(r)?;
        let jb: Value = serde_json::from_str(r2)?;
        let c = convert(&ja);
        let c2 = diff_rec(&ja, &jb);

        println!("{}", r2);
        println!("{:?}", c);
        println!("{:?}", c2);
        Ok(())
    }
}
