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
    ObjectValueDiff(&'a str, DiffType2<'a>),
    // Element(JsonV<'a>, JsonVPair<'a>),
    // Primitive(JsonVPair<'a>),
    // DifferentTypes(JsonVPair<'a>),
}

#[derive(Clone, Debug)]
enum DiffType2<'a> {
    Primitive(JsonVPair<'a>),
    DifferentTypes(JsonVPair<'a>),
    ObjectValueDiff(JsonV<'a>),
}

#[derive(Clone, Debug)]
enum Either<L: std::clone::Clone, R: std::clone::Clone> {
    Left(L),
    Right(R),
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

fn diff_rec<'a>(a0: &'a Value, b0: &'a Value) -> Either<JsonV<'a>, DiffType2<'a>> {
    match (a0, b0) {
        // Check keys first then values
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            // Find fields not in the other and vice versa
            let a_keys: Vec<_> = a_obj.keys().collect();
            let b_keys: Vec<_> = b_obj.keys().collect();
            let fields_in_b_not_a = a_intersection_complement_b(b_keys.clone(), a_keys.clone());
            let fields_in_a_not_b = a_intersection_complement_b(a_keys.clone(), b_keys.clone());

            // Find fields where the values differ in the two structures

            let mut differences: Vec<DiffType> = Vec::new();
            let mut similarities: HashMap<String, JsonV> = HashMap::new();
            for key in union(a_keys.clone(), b_keys.clone()) {
                if let (Some(a_value), Some(b_value)) = (a_obj.get(key), b_obj.get(key)) {
                    match diff_rec(a_value, b_value) {
                        Either::Left(json_element) if get_status(&json_element).is_some() => {
                            differences.push(DiffType::ObjectValueDiff(
                                key,
                                DiffType2::ObjectValueDiff(json_element),
                            ));
                        }
                        Either::Left(json_element) => {
                            similarities.insert(key.to_string(), json_element);
                        }
                        Either::Right(status) => {
                            differences.push(DiffType::ObjectValueDiff(key, status));
                        }
                    }
                }
            }

            let df = differences.clone();

            let status = if fields_in_a_not_b.is_empty()
                && fields_in_b_not_a.is_empty()
                && differences.is_empty()
            {
                // Objects are the same
                None
            } else {
                // Not the same
                let f1 = |x| DiffType::ObjectKeyPresent(x, convert(a_obj.get(x).unwrap()));
                let f2 = |x| DiffType::ObjectKeyMissing(x, convert(b_obj.get(x).unwrap()));
                let mut v1: Vec<DiffType> = fields_in_a_not_b.iter().map(|y| f1(y)).collect();
                let v2: Vec<DiffType> = fields_in_b_not_a.iter().map(|y| f2(y)).collect();

                v1.extend(v2);
                v1.extend(differences);

                Some(Status {
                    different_values: v1,
                })
            };
            let res = JsonV::Object(similarities, status);

            println!("0. {:?} {:?}", a0, b0);
            println!("2. {:?}", fields_in_a_not_b);
            println!("3. {:?}", fields_in_b_not_a);
            println!("4. {:?}", df);
            Either::Left(res)
        }
        // Check equal number of elements and element equality
        (Value::Array(p1), Value::Array(p2)) => Either::Left(JsonV::Null(None)),
        (Value::String(p1), Value::String(p2)) => {
            let k = if p1 == p2 {
                Either::Left(JsonV::String(p1.to_string(), None))
            } else {
                let v1 = DiffType2::Primitive((convert(a0), convert(b0)));
                Either::Right(v1)
            };
            k
        }
        (Value::Number(p1), Value::Number(p2)) => {
            println!("6. {:?} {:?}", p1, p2);
            let k = if cmp_option(p1.as_f64(), p2.as_f64()) {
                Either::Left(JsonV::Number(p1.as_f64().unwrap(), None))
            } else {
                println!("8. {:?} {:?}", p1, p2);
                let v1 = DiffType2::Primitive((convert(a0), convert(b0)));
                Either::Right(v1)
            };
            k
        }
        (Value::Bool(p1), Value::Bool(p2)) => {
            let k = if p1 == p2 {
                Either::Left(JsonV::Bool(*p1, None))
            } else {
                let v1 = DiffType2::Primitive((JsonV::Bool(*p1, None), JsonV::Bool(*p2, None)));
                Either::Right(v1)
            };
            k
        }
        (Value::Null, Value::Null) => Either::Left(JsonV::Null(None)),
        _ => Either::Right(DiffType2::DifferentTypes((convert(a0), convert(b0)))),
    }
}

fn cmp_option<T: std::string::ToString>(a: Option<T>, b: Option<T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            println!("7. {:?} {:?}", a.to_string(), b.to_string());
            a.to_string() == b.to_string()
        }
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

fn get_status<'a>(j: &'a JsonV) -> Option<Status<'a>> {
    match j {
        JsonV::Null(st) => st.clone(),
        JsonV::String(_, st) => st.clone(),
        JsonV::Bool(_, st) => st.clone(),
        JsonV::Number(_, st) => st.clone(),
        JsonV::Object(_, st) => st.clone(),
        JsonV::Array(_, st) => st.clone(),
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
            "f8": "v1",
            "f3": false,
            "f2": {
                "f3": 123,
                "f4": 456
            }
        }"#;
        let r2 = r#"
            {
            "f1": "v2",
            "f8": "v2",
            "f3": true,
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
            "f8": "v0",
            "f9": false,
            "f2": {
                "f3": 456,
                "f4": {
                    "f0":123
                }
            },
            "f3": {
                "f5": "v6"
            }
        }"#;
        let r2 = r#"
            {
            "f1": "v2",
            "f8": "v1",
            "f9": true,
            "f2": {
                "f3": 456,
                "f4": {
                    "f0":122
                }
            },
            "f3": "v6"
        }"#;

        println!("{}", r);
        println!("{}", r2);

        // diff(r, r2);
        let ja: Value = serde_json::from_str(r)?;
        let jb: Value = serde_json::from_str(r2)?;
        let c = convert(&ja);
        let c2 = diff_rec(&ja, &jb);

        // println!("{}", r2);
        // println!("{:?}", c);
        println!("");
        println!("{:?}", c2);
        Ok(())
    }
}
