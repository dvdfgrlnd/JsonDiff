use serde_json::{Result, Value};
use std::collections::HashMap;

mod edit_distance;
use edit_distance::{edit_distance, EditType};

#[derive(Clone, Debug)]
enum JsonV<'a> {
    Null(Option<Status<'a>>),
    String(String, Option<Status<'a>>),
    Bool(bool, Option<Status<'a>>),
    Number(f64, Option<Status<'a>>),
    Array(Vec<(usize, JsonV<'a>)>, Option<Status<'a>>),
    Object(HashMap<String, JsonV<'a>>, Option<Status<'a>>),
}

type JsonVPair<'a> = (JsonV<'a>, JsonV<'a>);

#[derive(Clone, Debug)]
enum DiffType<'a> {
    ObjectKeyMissing(&'a str, JsonV<'a>),
    ObjectKeyPresent(&'a str, JsonV<'a>),
    ObjectValueDiff(&'a str, DiffType2<'a>),
    ArrayValueInSecond(usize, JsonV<'a>),
    ArrayValueInFirst(usize, JsonV<'a>),
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

fn main() -> Result<()> {
    println!("Hello, world!");

    diff("", "")
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

            // let df = differences.clone();

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

            Either::Left(res)
        }
        // Check equal number of elements and element equality
        (Value::Array(arr1), Value::Array(arr2)) => {
            let arr1_values: Vec<String> = arr1.iter().map(|x| x.to_string()).collect();
            let arr2_values: Vec<String> = arr2.iter().map(|x| x.to_string()).collect();

            let mut res = edit_distance(arr1_values, arr2_values);
            res.reverse();
            let mut same: Vec<(usize, JsonV)> = Vec::new();
            let mut diffs: Vec<DiffType> = Vec::new();
            let mut i: usize = 0;
            for x in res {
                match x {
                    EditType::Insert(y) => {
                        diffs.push(DiffType::ArrayValueInSecond(i, convert(&arr2[y])))
                    }
                    EditType::Delete(y) => {
                        diffs.push(DiffType::ArrayValueInFirst(i, convert(&arr1[y])))
                    }
                    EditType::Substitute(y, _, is_same) if is_same => {
                        same.push((i, convert(&arr1[y])))
                    }
                    EditType::Substitute(y, z, _) => {
                        diffs.push(DiffType::ArrayValueInSecond(i, convert(&arr2[z])));
                        diffs.push(DiffType::ArrayValueInFirst(i, convert(&arr1[y])));
                    }
                    EditType::Unknown => (),
                }
                i += 1;
            }
            let st = if diffs.is_empty() {
                None
            } else {
                Some(Status{different_values: diffs})
            };

            Either::Left(JsonV::Array(same, st))
        }
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
            let na = b.iter().enumerate().map(|(i, v)| (i, convert(v))).collect();
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

    #[test]
    fn test_edit_distance() -> Result<()> {
        let res = edit_distance("2334".chars().collect(), "1223344".chars().collect());
        println!("{:?}", res);
        Ok(())
    }
}
