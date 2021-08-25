use super::json_diff::{ArrayDiff, JsonV, ObjectDiff};
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
enum Line {
    Same(usize, String),
    DiffMissing(usize, String),
    DiffPresent(usize, String),
    NewLine,
    Text(usize, String),
    Start,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self {
            Line::Same(indent, x) => x,
            Line::DiffMissing(indent, x) => x,
            Line::DiffPresent(indent, x) => x,
            Line::Text(indent, s) => s,
            Line::Start => "",
            Line::NewLine => "\n",
        };
        write!(f, "{}", v)
    }
}

#[derive(Debug, Clone)]
struct Node {
    content: Line,
    previous: Option<Box<Node>>,
}

pub fn generate(json: JsonV) -> String {
    let mut acc = Node {
        previous: None,
        content: Line::Start,
    };
    acc = generate_rec(0, json, acc, None);

    // println!("{:?}\n", acc);

    let sum = node_to_string(acc.clone());
    println!("{}", sum);

    let html = to_html(acc);
    // println!("{}", html);

    html
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

fn generate_rec(
    indent: usize,
    json: JsonV,
    acc: Node,
    type_to_use: Option<fn(usize, String) -> Line>,
) -> Node {
    // println!("{:?}", json);
    match json {
        JsonV::Null(st) => {
            if let Some(o) = st {
                let n1 = generate_rec(indent, o.0, acc, Some(|i, x| Line::DiffPresent(i, x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(indent, ", ".to_string()),
                };
                let n3 = generate_rec(indent, o.1, n2, Some(|i, x| Line::DiffMissing(i, x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(
                        indent,
                        "null".to_string(),
                    ),
                }
            }
        }
        JsonV::String(s, st) => {
            if let Some(o) = st {
                let n1 = generate_rec(indent, o.0, acc, Some(|i, x| Line::DiffPresent(i, x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(indent, ", ".to_string()),
                };
                let n3 = generate_rec(indent, o.1, n2, Some(|i, x| Line::DiffMissing(i, x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(
                        indent,
                        format!("\"{}\"", s),
                    ),
                }
            }
        }
        JsonV::Bool(b, st) => {
            let bool_string = if b { "true" } else { "false" };
            if let Some(o) = st {
                let n1 = generate_rec(indent, o.0, acc, Some(|i, x| Line::DiffPresent(i, x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(indent, ", ".to_string()),
                };
                let n3 = generate_rec(indent, o.1, n2, Some(|i, x| Line::DiffMissing(i, x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(
                        indent,
                        bool_string.to_string(),
                    ),
                }
            }
        }
        JsonV::Number(n, st) => {
            if let Some(o) = st {
                let n1 = generate_rec(indent, o.0, acc, Some(|i, x| Line::DiffPresent(i, x)));
                let n2 = Node {
                    previous: Some(Box::new(n1)),
                    content: Line::Text(indent, ", ".to_string()),
                };
                let n3 = generate_rec(indent, o.1, n2, Some(|i, x| Line::DiffMissing(i, x)));
                n3
            } else {
                Node {
                    previous: Some(Box::new(acc)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(indent, n.to_string()),
                }
            }
        }
        JsonV::Array(v, st) => {
            let mut n = Node {
                previous: Some(Box::new(acc)),
                content: Line::Text(indent, "[".to_string()),
            };
            n = Node {
                previous: Some(Box::new(n)),
                content: Line::NewLine,
            };
            for (_, element) in v {
                n = generate_rec(indent + 1, element, n, type_to_use);
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(indent + 1, ", ".to_string()),
                };
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::NewLine,
                };
            }
            for s in st {
                match s {
                    ArrayDiff::ArrayValueInFirst(_, el) => {
                        n = generate_rec(indent + 1, el, n, Some(|i, x| Line::DiffPresent(i, x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, ", ".to_string()),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                    }
                    ArrayDiff::ArrayValueInSecond(_, el) => {
                        n = generate_rec(indent + 1, el, n, Some(|i, x| Line::DiffMissing(i, x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, ", ".to_string()),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                    }
                }
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::NewLine,
                };
            }

            n = Node {
                previous: Some(Box::new(n)),
                content: Line::Same(indent, "]".to_string()),
            };
            Node {
                previous: Some(Box::new(n)),
                content: Line::NewLine,
            }
        }
        JsonV::Object(h, st) => {
            let mut n = Node {
                previous: Some(Box::new(acc)),
                content: Line::Text(indent, "{".to_string()),
            };
            n = Node {
                previous: Some(Box::new(n)),
                content: Line::NewLine,
            };
            for (k, v) in h {
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(indent + 1, format!("{}: ", k)),
                };
                n = generate_rec(indent + 1, v, n, type_to_use);
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::Text(indent + 1, ", ".to_string()),
                };
                n = Node {
                    previous: Some(Box::new(n)),
                    content: Line::NewLine,
                };
            }
            for m in st {
                match m {
                    ObjectDiff::ObjectKeyMissing(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, format!("{}: ", s)),
                        };
                        n = generate_rec(indent + 1, v, n, Some(|i, x| Line::DiffMissing(i, x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, ", ".to_string()),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                    }
                    ObjectDiff::ObjectKeyPresent(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, format!("{}: ", s)),
                        };
                        n = generate_rec(indent + 1, v, n, Some(|i, x| Line::DiffPresent(i, x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, ", ".to_string()),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                    }
                    ObjectDiff::ObjectValueDiff(s, v) => {
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, format!("{}: ###", s)),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                        n = generate_rec(indent + 1, v, n, Some(|i, x| Line::DiffPresent(i, x)));
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::Text(indent + 1, "###, ".to_string()),
                        };
                        n = Node {
                            previous: Some(Box::new(n)),
                            content: Line::NewLine,
                        };
                    }
                }
            }
            n = Node {
                previous: Some(Box::new(n)),
                content: Line::Text(indent, "}".to_string()),
            };
            Node {
                previous: Some(Box::new(n)),
                content: Line::NewLine,
            }
        }
    }
}

enum L2 {
    Text(String),
    Newline,
}

fn to_html(n: Node) -> String {
    let mut n = Some(Box::new(n));
    let mut sum: Vec<Vec<String>> = Vec::new();
    let mut curr: Vec<String> = Vec::new();
    while n.is_some() {
        if let Some(n2) = n {
            if let Some(x) = generate_line(n2.content) {
                match x {
                    L2::Text(t) => curr.push(t),
                    L2::Newline => {
                        curr.reverse();
                        sum.push(curr);
                        curr = Vec::new();
                    }
                }
            }

            n = n2.previous;
        }
    }
    if curr.len() > 0 {
        sum.push(curr);
    }
    sum.reverse();
    let mut r = "".to_string();
    for v in sum {
        r.push_str(&format!("<div>{}</div>", v.join("")));
    }
    format!("<div style=\"display: flex; flex-direction: column;\">{}</div>", r)
}

fn generate_line(n: Line) -> Option<L2> {
    match n {
        Line::DiffMissing(indent, x) => Some(L2::Text(missing(indent, x))),
        Line::DiffPresent(indent, x) => Some(L2::Text(present(indent, x))),
        Line::NewLine => Some(L2::Newline),
        Line::Same(indent, x) => Some(L2::Text(same(indent, x))),
        Line::Start => None,
        Line::Text(indent, s) => Some(L2::Text(same(indent, s))),
    }
}

fn same(indent: usize, s: String) -> String {
    format!(
        "<span class=\"same\" style=\"margin-left:{}px;color:black\">{}</span>\n",
        (30 * indent).to_string(),
        s
    )
    .to_string()
}

fn present(indent: usize, s: String) -> String {
    format!(
        "<span class=\"present\" style=\"margin-left:{}px;color:green\">+++{}+++</span>\n",
        (30 * indent).to_string(),
        s
    )
    .to_string()
}

fn missing(indent: usize, s: String) -> String {
    format!(
        "<span class=\"missing\" style=\"margin-left:{}px;color:red\">---{}---</span>\n",
        (30 * indent).to_string(),
        s
    )
    .to_string()
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
                vec![ArrayDiff::ArrayValueInFirst(
                    0,
                    JsonV::String("arr".to_string(), None),
                )],
            ),
        );

        let json = JsonV::Object(h, Vec::new());
        println!("{:?}\n", json);
        let res = generate(json);
        println!("{}\n", res);

        println!("{}", res);
    }
}
