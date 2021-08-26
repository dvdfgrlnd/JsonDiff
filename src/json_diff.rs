use serde_json::{Result, Value};
use std::collections::HashMap;

use super::edit_distance;
use edit_distance::{edit_distance, EditType};

#[derive(Clone, Debug)]
pub enum JsonV {
    Null(Option<Box<JsonVPair>>),
    String(String, Option<Box<JsonVPair>>),
    Bool(bool, Option<Box<JsonVPair>>),
    Number(f64, Option<Box<JsonVPair>>),
    Array(Vec<(usize, JsonV)>, Vec<ArrayDiff>),
    Object(HashMap<String, JsonV>, Vec<ObjectDiff>),
}

pub type JsonVPair = (JsonV, JsonV);

#[derive(Clone, Debug)]
pub enum ArrayDiff {
    ArrayValueInSecond(usize, JsonV),
    ArrayValueInFirst(usize, JsonV),
}

#[derive(Clone, Debug)]
pub enum ObjectDiff {
    ObjectKeyMissing(String, JsonV),
    ObjectKeyPresent(String, JsonV),
    ObjectValueDiff(String, JsonV),
}

pub fn diff(a: &str, b: &str) -> Result<JsonV> {
    let a_as_json: Value = serde_json::from_str(a)?;
    let b_as_json: Value = serde_json::from_str(b)?;

    let json = diff_rec(&a_as_json, &b_as_json);

    Ok(json)
}

fn diff_rec(arg1: &Value, arg2: &Value) -> JsonV {
    match (arg1, arg2) {
        // Check keys first then values
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            let a_keys: Vec<_> = a_obj.keys().map(|x| x.to_string()).collect::<Vec<String>>();
            let b_keys: Vec<_> = b_obj.keys().map(|x| x.to_string()).collect::<Vec<String>>();
            // Find fields not in the other and vice versa
            let fields_in_a_not_b = a_intersection_complement_b(a_keys.clone(), b_keys.clone());
            let fields_in_b_not_a = a_intersection_complement_b(b_keys.clone(), a_keys.clone());

            // Find fields where the values differ in the two structures

            let mut differences: Vec<ObjectDiff> = Vec::new();
            let mut similarities: HashMap<String, JsonV> = HashMap::new();
            for key in union(a_keys.clone(), b_keys.clone()) {
                if let (Some(a_value), Some(b_value)) = (a_obj.get(&key), b_obj.get(&key)) {
                    let json_element = diff_rec(a_value, b_value);
                    if has_differences(&json_element) {
                        differences.push(ObjectDiff::ObjectValueDiff(key, json_element));
                    } else {
                        similarities.insert(key.to_string(), json_element);
                    }
                }
            }

            // let df = differences.clone();

            let all_differences = if fields_in_a_not_b.is_empty()
                && fields_in_b_not_a.is_empty()
                && differences.is_empty()
            {
                // Objects are the same
                Vec::new()
            } else {
                // Not the same
                let entries_present_in_a: Vec<ObjectDiff> = fields_in_a_not_b
                    .iter()
                    .map(|x| {
                        ObjectDiff::ObjectKeyPresent(x.to_string(), convert(a_obj.get(x).unwrap()))
                    })
                    .collect();
                let entries_missing_in_a: Vec<ObjectDiff> = fields_in_b_not_a
                    .iter()
                    .map(|x| {
                        ObjectDiff::ObjectKeyMissing(x.to_string(), convert(b_obj.get(x).unwrap()))
                    })
                    .collect();

                [entries_present_in_a, entries_missing_in_a, differences].concat()
            };
            JsonV::Object(similarities, all_differences)
        }
        // Check equal number of elements and element equality
        (Value::Array(arr1), Value::Array(arr2)) => {
            let arr1_elements_serialized: Vec<String> =
                arr1.iter().map(|x| x.to_string()).collect();
            let arr2_elements_serialized: Vec<String> =
                arr2.iter().map(|x| x.to_string()).collect();

            let edit_types = edit_distance(arr1_elements_serialized, arr2_elements_serialized);
            let mut same: Vec<(usize, JsonV)> = Vec::new();
            let mut diffs: Vec<ArrayDiff> = Vec::new();
            for (i, edit_type) in edit_types.iter().enumerate() {
                match edit_type {
                    EditType::Insert(index) => {
                        diffs.push(ArrayDiff::ArrayValueInSecond(i, convert(&arr2[*index])))
                    }
                    EditType::Delete(index) => {
                        diffs.push(ArrayDiff::ArrayValueInFirst(i, convert(&arr1[*index])))
                    }
                    EditType::Substitute(index, _, is_same) if *is_same => {
                        same.push((i, convert(&arr1[*index])))
                    }
                    EditType::Substitute(index_arg1, index_arg2, _) => {
                        diffs.push(ArrayDiff::ArrayValueInSecond(
                            i,
                            convert(&arr2[*index_arg2]),
                        ));
                        diffs.push(ArrayDiff::ArrayValueInFirst(i, convert(&arr1[*index_arg1])));
                    }
                    EditType::Unknown => (),
                }
            }

            JsonV::Array(same, diffs)
        }
        (Value::String(s1), Value::String(s2)) => {
            let k = if s1 == s2 {
                JsonV::String(s1.to_string(), None)
            } else {
                let v1 = Box::new((convert(arg1), convert(arg2)));
                JsonV::String("".to_string(), Some(v1))
            };
            k
        }
        (Value::Number(n1), Value::Number(n2)) => {
            if cmp_option(n1.as_f64(), n2.as_f64()) {
                JsonV::Number(n1.as_f64().unwrap(), None)
            } else {
                JsonV::Number(0.0, Some(Box::new((convert(arg1), convert(arg2)))))
            }
        }
        (Value::Bool(b1), Value::Bool(b2)) => {
            if b1 == b2 {
                JsonV::Bool(*b1, None)
            } else {
                JsonV::Bool(
                    false,
                    Some(Box::new((JsonV::Bool(*b1, None), JsonV::Bool(*b2, None)))),
                )
            }
        }
        (Value::Null, Value::Null) => JsonV::Null(None),
        _ => JsonV::Null(Some(Box::new((convert(arg1), convert(arg2))))),
    }
}

fn cmp_option<T: std::string::ToString>(a: Option<T>, b: Option<T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            // println!("7. {:?} {:?}", a.to_string(), b.to_string());
            a.to_string() == b.to_string()
        }
        (None, None) => false,
        _ => false,
    }
}

fn convert(v: &Value) -> JsonV {
    match v {
        Value::Null => JsonV::Null(None),
        Value::Bool(b) => JsonV::Bool(*b, None),
        Value::Number(n) => JsonV::Number(n.as_f64().unwrap_or(0.0), None),
        Value::String(s) => JsonV::String(s.to_string(), None),
        Value::Object(o) => {
            let mut map: HashMap<String, JsonV> = HashMap::new();
            for k in o.keys() {
                if let Some(v) = o.get(k) {
                    map.insert(k.to_string(), convert(v));
                }
            }
            JsonV::Object(map, Vec::new())
        }
        Value::Array(b) => {
            let converted_elements = b.iter().enumerate().map(|(i, v)| (i, convert(v))).collect();
            JsonV::Array(converted_elements, Vec::new())
        }
    }
}

fn has_differences(j: &JsonV) -> bool {
    match j {
        JsonV::Null(st) if st.is_some() => true,
        JsonV::String(_, st) if st.is_some() => true,
        JsonV::Bool(_, st) if st.is_some() => true,
        JsonV::Number(_, st) if st.is_some() => true,
        JsonV::Object(_, st) if st.is_empty() => true,
        JsonV::Array(_, st) if st.is_empty() => true,
        _ => false,
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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

        let ja: Value = serde_json::from_str(r)?;
        let jb: Value = serde_json::from_str(r2)?;
        let c2 = diff_rec(&ja, &jb);

        println!("");
        println!("{:?}", c2);
        Ok(())
    }

    #[test]
    fn test_diff_3() -> Result<()> {
        let r = r#"
        {
            "f0": "test",
            "f1":
                [
                    {"item1": 123}, 
                    {"item2": 123}, 
                    {"item3": 345}, 
                    {"item4": 345} 
                ]
        }
        "#;
        let r2 = r#"
        {
            "f0": "test",
            "f1":
                [
                    {"item0": 123}, 
                    {"item2": 123}, 
                    {"item3": 123}, 
                    {"item5": 345}
                ]
        }
        "#;

        println!("{}", r);
        println!("{}", r2);

        // diff(r, r2);
        let ja: Value = serde_json::from_str(r)?;
        let jb: Value = serde_json::from_str(r2)?;
        let c2 = diff_rec(&ja, &jb);

        println!("");
        println!("{:?}", c2);
        Ok(())
    }
}
