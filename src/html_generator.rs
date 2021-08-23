use super::json_diff::{ArrayDiff, JsonV, ObjectDiff};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
enum Line {
    Same(String),
    DiffMissing(String),
    DiffPresent(String),
    NewLine,
    Text(String),
    Start,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self {
            Line::Same(x) => x,
            Line::DiffMissing(x) => x,
            Line::DiffPresent(x) => x,
            Line::Text(s) => s,
            Line::Start => "",
            Line::NewLine => "\n",
        };
        write!(f, "{}", v)
    }
}

#[derive(Debug)]
struct Node {
    content: Line,
    previous: Option<Box<Node>>,
}

pub fn generate(json: JsonV) -> String {
    let mut acc = Node {
        previous: None,
        content: Line::Start,
    };
    acc = generate_rec(json, acc, None);

    println!("{:?}\n", acc);

    let sum = node_to_string(acc);
    println!("{}", sum);

    // acc.join("\n")
    "".to_string()
}

fn node_to_string(acc: Node) -> String {
    let mut n = Some(Box::new(acc));
    let mut sum = "".to_string();
    while n.is_some() {
        if let Some(n2) = n {
            sum = n2.content.to_string() + &sum;

            n = n2.previous;
        }
    }
    sum
}

fn generate_rec(json: JsonV, acc: Node, type_to_use: Option<fn(String) -> Line>) -> Node {
    // println!("{:?}", json);
    match json {
        JsonV::Null(st) => {
            if let Some(o) = st {
                let n1 = generate_rec(o.0, acc, Some(|x| Line::DiffPresent(x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(", ".to_string()),
                };
                let n3 = generate_rec(o.1, n2, Some(|x| Line::DiffMissing(x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|x| Line::Same(x))("null".to_string()),
                }
            }
        }
        JsonV::String(s, st) => {
            if let Some(o) = st {
                let n1 = generate_rec(o.0, acc, Some(|x| Line::DiffPresent(x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(", ".to_string()),
                };
                let n3 = generate_rec(o.1, n2, Some(|x| Line::DiffMissing(x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|x| Line::Same(x))(format!("\"{}\"", s)),
                }
            }
        }
        JsonV::Bool(b, st) => {
            let bool_string = if b { "true" } else { "false" };
            if let Some(o) = st {
                let n1 = generate_rec(o.0, acc, Some(|x| Line::DiffPresent(x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(", ".to_string()),
                };
                let n3 = generate_rec(o.1, n2, Some(|x| Line::DiffMissing(x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|x| Line::Same(x))(bool_string.to_string()),
                }
            }
        }
        JsonV::Number(n, st) => {
            if let Some(o) = st {
                let n1 = generate_rec(o.0, acc, Some(|x| Line::DiffPresent(x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(", ".to_string()),
                };
                let n3 = generate_rec(o.1, n2, Some(|x| Line::DiffMissing(x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|x| Line::Same(x))(n.to_string()),
                }
            }
        }
        JsonV::Array(v, st) => {
            let mut n = Node {
                previous: Some(Box::new(acc)),
                content: Line::Text("[".to_string()),
            };
            for (_, element) in v {
                n = generate_rec(element, n, type_to_use);
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(", ".to_string()),
                };
            }
            for s in st {
                match s {
                    ArrayDiff::ArrayValueInFirst(_, el) => {
                        n = generate_rec(el, n, Some(|x| Line::DiffPresent(x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(", ".to_string()),
                        };
                    }
                    ArrayDiff::ArrayValueInSecond(_, el) => {
                        n = generate_rec(el, n, Some(|x| Line::DiffMissing(x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(", ".to_string()),
                        };
                    }
                }
            }
            Node {
                previous: Some(Box::new(n)),
                content: Line::Same("]".to_string()),
            }
        }
        JsonV::Object(h, st) => {
            let mut n = Node {
                previous: Some(Box::new(acc)),
                content: Line::Text("{".to_string()),
            };
            for (k, v) in h {
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(format!("{}: ", k)),
                };
                n = generate_rec(v, n, type_to_use);
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(", ".to_string()),
                };
            }
            for m in st {
                match m {
                    ObjectDiff::ObjectKeyMissing(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(format!("{}: ", s)),
                        };
                        n = generate_rec(v, n, Some(|x| Line::DiffMissing(x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(", ".to_string()),
                        };
                    }
                    ObjectDiff::ObjectKeyPresent(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(format!("{}: ", s)),
                        };
                        n = generate_rec(v, n, Some(|x| Line::DiffPresent(x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(", ".to_string()),
                        };
                    }
                    ObjectDiff::ObjectValueDiff(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(format!("{}: ###", s)),
                        };
                        n = generate_rec(v, n, Some(|x| Line::DiffPresent(x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text("###, ".to_string()),
                        };
                    }
                }
            }
            Node {
                previous: Some(Box::new(n)),
                content: Line::Text("}".to_string()),
            }
        }
    }
}

fn generate_line(mut acc: String, add: &str) -> String {
    acc.push_str(&format!("<p class=\"line\">{}<\\p>\n", add).to_string());

    acc
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_generate_html() {
        let mut h: HashMap<String, JsonV> = HashMap::new();
        let mut h2: HashMap<String, JsonV> = HashMap::new();
        h2.insert(
            "item5".to_string(),
            JsonV::String("value5".to_string(), None),
        );

        h.insert(
            "item1".to_string(),
            JsonV::String("value1".to_string(), None),
        );
        h.insert(
            "item2".to_string(),
            JsonV::String("value2".to_string(), None),
        );
        h.insert(
            "item3".to_string(),
            JsonV::Object(
                h2,
                vec![ObjectDiff::ObjectValueDiff(
                    "missing1".to_string(),
                    JsonV::Bool(
                        true,
                        Some(Box::new((
                            JsonV::Bool(false, None),
                            JsonV::Bool(true, None),
                        ))),
                    ),
                )],
            ),
        );
        h.insert(
            "item5".to_string(),
            JsonV::Array(
                vec![
                    (0, JsonV::Number(10.0, None)),
                    (1, JsonV::Number(20.0, None)),
                ],
                vec![
                    ArrayDiff::ArrayValueInFirst(0, JsonV::String("arr".to_string(), None))
                    ],
            ),
        );

        let json = JsonV::Object(h, Vec::new());
        println!("{:?}\n", json);
        let res = generate(json);
        println!("{}\n", res);

        println!("{}", res);
    }
}
