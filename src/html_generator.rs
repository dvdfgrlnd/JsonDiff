use super::json_diff::{ArrayDiff, JsonV, ObjectDiff};
use std::clone::Clone;
use std::collections::BTreeMap;
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
        let s = match self {
            Line::Same(_, x) => x,
            Line::DiffMissing(_, x) => x,
            Line::DiffPresent(_, x) => x,
            Line::Text(_, s) => s,
            Line::Start => "",
            Line::NewLine => "\n",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Debug, Clone)]
struct Node {
    content: Line,
    previous: Option<Box<Node>>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut curr_node = Some(Box::new(self.clone()));
        let mut agg = "".to_string();
        while curr_node.is_some() {
            if let Some(node) = curr_node {
                agg = node.content.to_string() + &agg;

                curr_node = node.previous;
            }
        }
        write!(f, "{}", agg)
    }
}

pub fn generate(json: JsonV) -> String {
    let start_node = Node {
        previous: None,
        content: Line::Start,
    };
    let last_node = generate_rec(0, json, start_node, None);

    let html = to_html(last_node);

    html
}

fn text_node(previous: Node, text: String, indent: usize) -> Node {
    Node {
        previous: Some(Box::new(previous)),
        content: Line::Text(indent, text),
    }
}

fn newline_node(previous: Node) -> Node {
    Node {
        previous: Some(Box::new(previous)),
        content: Line::NewLine,
    }
}

fn generate_rec(
    indent: usize,
    json: JsonV,
    last_node: Node,
    type_to_use: Option<fn(usize, String) -> Line>,
) -> Node {
    match json {
        JsonV::Null(st) => {
            if let Some(o) = st {
                let mut curr_node =
                    generate_rec(indent, o.0, last_node, Some(|i, x| Line::DiffPresent(i, x)));
                curr_node = text_node(curr_node, ", ".to_string(), indent);
                curr_node = newline_node(curr_node);
                curr_node =
                    generate_rec(indent, o.1, curr_node, Some(|i, x| Line::DiffMissing(i, x)));
                curr_node
            } else {
                Node {
                    previous: Some(Box::new(last_node)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(
                        indent,
                        "null".to_string(),
                    ),
                }
            }
        }
        JsonV::String(s, st) => {
            if let Some(o) = st {
                let mut curr_node =
                    generate_rec(indent, o.0, last_node, Some(|i, x| Line::DiffPresent(i, x)));
                curr_node = text_node(curr_node, ", ".to_string(), indent);
                curr_node =
                    generate_rec(indent, o.1, curr_node, Some(|i, x| Line::DiffMissing(i, x)));
                curr_node
            } else {
                Node {
                    previous: Some(Box::new(last_node)),
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
                let mut curr_node =
                    generate_rec(indent, o.0, last_node, Some(|i, x| Line::DiffPresent(i, x)));
                curr_node = text_node(curr_node, ", ".to_string(), indent);
                curr_node =
                    generate_rec(indent, o.1, curr_node, Some(|i, x| Line::DiffMissing(i, x)));
                curr_node
            } else {
                Node {
                    previous: Some(Box::new(last_node)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(
                        indent,
                        bool_string.to_string(),
                    ),
                }
            }
        }
        JsonV::Number(n, st) => {
            if let Some(o) = st {
                let mut curr_node =
                    generate_rec(indent, o.0, last_node, Some(|i, x| Line::DiffPresent(i, x)));
                curr_node = text_node(curr_node, ", ".to_string(), indent);
                curr_node =
                    generate_rec(indent, o.1, curr_node, Some(|i, x| Line::DiffMissing(i, x)));
                curr_node
            } else {
                Node {
                    previous: Some(Box::new(last_node)),
                    content: type_to_use.unwrap_or(|i, x| Line::Same(i, x))(indent, n.to_string()),
                }
            }
        }
        JsonV::Array(v, differences) => {
            let mut curr_node = text_node(last_node, "[".to_string(), indent);
            curr_node = Node {
                previous: Some(Box::new(curr_node)),
                content: Line::NewLine,
            };
            let elements = get_array_elements_in_order(v, differences);
            for e in elements {
                let (element, type_of_line): (JsonV, fn(usize, String) -> Line) = match e {
                    Either::Left((_, element)) => (element, type_to_use.unwrap_or(|i, x| Line::Same(i, x))),
                    Either::Right(ArrayDiff::ArrayValueInFirst(_, element)) => (element, |i, x| Line::DiffPresent(i, x)),
                    Either::Right(ArrayDiff::ArrayValueInSecond(_, element)) => (element, |i, x| Line::DiffMissing(i, x)),
                };
                curr_node = generate_rec(
                    indent + 1,
                    element,
                    curr_node,
                    Some(type_of_line),
                );
                curr_node = text_node(curr_node, ", ".to_string(), indent);
                curr_node = newline_node(curr_node);
            }

            text_node(curr_node, "]".to_string(), indent)
        }
        JsonV::Object(h, st) => {
            let mut curr_node = text_node(last_node, "{".to_string(), indent);
            curr_node = newline_node(curr_node);
            for (k, v) in h {
                curr_node = text_node(curr_node, format!("{}: ", k), indent + 1);
                curr_node = generate_rec(indent + 1, v, curr_node, type_to_use);
                curr_node = text_node(curr_node, ", ".to_string(), indent + 1);
                curr_node = newline_node(curr_node);
            }
            for m in st {
                match m {
                    ObjectDiff::ObjectKeyMissing(s, v) => {
                        curr_node = text_node(curr_node, format!("{}: ", s), indent + 1);
                        curr_node = generate_rec(
                            indent + 1,
                            v,
                            curr_node,
                            Some(|i, x| Line::DiffMissing(i, x)),
                        );
                        curr_node = text_node(curr_node, ", ".to_string(), indent + 1);
                        curr_node = newline_node(curr_node);
                    }
                    ObjectDiff::ObjectKeyPresent(s, v) => {
                        curr_node = text_node(curr_node, format!("{}: ", s), indent + 1);
                        curr_node = generate_rec(
                            indent + 1,
                            v,
                            curr_node,
                            Some(|i, x| Line::DiffPresent(i, x)),
                        );
                        curr_node = text_node(curr_node, ", ".to_string(), indent + 1);
                        curr_node = newline_node(curr_node);
                    }
                    ObjectDiff::ObjectValueDiff(s, v) => {
                        let addon = if is_primitive_json_type(&v) {"###"} else {""};
                        curr_node = text_node(curr_node, format!("{}: {}", s, addon), indent + 1);
                        curr_node = newline_node(curr_node);
                        curr_node = generate_rec(
                            indent + 1,
                            v,
                            curr_node,
                            Some(|i, x| Line::Same(i, x)),
                        );
                        curr_node = newline_node(curr_node);
                        curr_node = text_node(curr_node, format!("{}, ", addon), indent + 1);
                        curr_node = newline_node(curr_node);
                    }
                }
            }
            text_node(curr_node, "}".to_string(), indent)
        }
    }
}

fn is_primitive_json_type(j: &JsonV) -> bool {
    match j {
        JsonV::Null(_) => true,
        JsonV::Bool(_, _) => true,
        JsonV::Number(_, _) => true,
        JsonV::String(_, _) => true,
        _ => false
    }
}

type SortNode = Either<(usize, JsonV), ArrayDiff>;

fn get_array_elements_in_order(
    v: Vec<(usize, JsonV)>,
    differences: Vec<ArrayDiff>,
) -> Vec<Either<(usize, JsonV), ArrayDiff>> {
    let mut all_elements: Vec<SortNode> = Vec::with_capacity(v.len() + differences.len());
    for e in v {
        all_elements.push(Either::Left(e));
    }

    for d in differences {
        all_elements.push(Either::Right(d));
    }
    let get_array_index = |a: &SortNode| -> usize {
        match a {
            Either::Left((i, _)) => *i,
            Either::Right(ArrayDiff::ArrayValueInFirst(i, _)) => *i,
            Either::Right(ArrayDiff::ArrayValueInSecond(i, _)) => *i,
        }
    };
    all_elements.sort_by(|a, b| get_array_index(a).partial_cmp(&get_array_index(b)).unwrap());

    all_elements
}

enum L2 {
    Text(String),
    Newline,
}

fn to_html(inp_node: Node) -> String {
    let mut maybe_node = Some(Box::new(inp_node));
    let mut lines: Vec<Vec<(usize, String)>> = Vec::new();
    let mut current_line: Vec<(usize, String)> = Vec::new();
    while maybe_node.is_some() {
        if let Some(node) = maybe_node {
            if let Some((indent, text_element)) = generate_line(node.content) {
                match text_element {
                    L2::Text(text) => current_line.push((indent, text)),
                    L2::Newline => {
                        if current_line.len() > 0 {
                            // Since inp_node start from the bottom-left the line should be flipped
                            current_line.reverse();
                            lines.push(current_line);
                            current_line = Vec::new();
                        }
                    }
                }
            }

            maybe_node = node.previous;
        }
    }
    if current_line.len() > 0 {
        lines.push(current_line);
    }
    // Since inp_node start from the bottom the lines should be reversed
    lines.reverse();
    let mut output_html = "".to_string();
    for single_line in lines {
        let maybe_indent = single_line
            .iter()
            .map(|x| x.0)
            .take(1)
            .collect::<Vec<usize>>()
            .pop();
        if let Some(indent) = maybe_indent {
            let concatenated_line_elements = single_line
                .iter()
                .map(|x| x.1.clone())
                .collect::<Vec<_>>()
                .join("");
            output_html.push_str(&format!(
                "<div style=\"margin-left:{}px\">{}</div>",
                (30 * indent).to_string(),
                concatenated_line_elements
            ));
        } else {
            panic!("No indent found!\n {:?}", single_line);
        }
    }
    format!(
        "<div style=\"display: flex; flex-direction: column;\">{}</div>",
        output_html
    )
}

fn generate_line(line: Line) -> Option<(usize, L2)> {
    match line {
        Line::DiffMissing(indent, text) => Some((indent, L2::Text(missing(text)))),
        Line::DiffPresent(indent, text) => Some((indent, L2::Text(present(text)))),
        Line::NewLine => Some((0, L2::Newline)),
        Line::Same(indent, text) => Some((indent, L2::Text(same(text)))),
        Line::Start => None,
        Line::Text(indent, text) => Some((indent, L2::Text(same(text)))),
    }
}

fn same(s: String) -> String {
    format!("<span class=\"same\" style=\"color:black\">{}</span>\n", s).to_string()
}

fn present(s: String) -> String {
    format!(
        "<span class=\"present\" style=\"color:green\">+++{}+++</span>\n",
        s
    )
    .to_string()
}

fn missing(s: String) -> String {
    format!(
        "<span class=\"missing\" style=\"color:red\">---{}---</span>\n",
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
        let mut map1: BTreeMap<String, JsonV> = BTreeMap::new();
        let mut map2: BTreeMap<String, JsonV> = BTreeMap::new();
        map2.insert(
            "item5".to_string(),
            JsonV::String("value5".to_string(), None),
        );

        map1.insert(
            "item1".to_string(),
            JsonV::String("value1".to_string(), None),
        );
        map1.insert(
            "item2".to_string(),
            JsonV::String("value2".to_string(), None),
        );
        map1.insert(
            "item3".to_string(),
            JsonV::Object(
                map2,
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
        map1.insert(
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

        let json = JsonV::Object(map1, Vec::new());
        println!("{:?}\n", json);
        let res = generate(json);

        println!("{}", res);
    }

    #[test]
    fn test_keep_array_order() {
        let arr = vec![(1, JsonV::String("test2".to_string(), None))];
        let d = vec![ArrayDiff::ArrayValueInFirst(
            0,
            JsonV::String("test1".to_string(), None),
        )];
        let res = get_array_elements_in_order(
            arr,
            d,
        );

        let exp: Vec<Either<(usize, JsonV), ArrayDiff>> = vec![
            Either::Right(ArrayDiff::ArrayValueInFirst(
                0,
                JsonV::String("test1".to_string(), None),
            )),
            Either::Left((1, JsonV::String("test2".to_string(), None))),
        ];

        println!("{}", format!("{:?}", res));
        println!("{}", format!("{:?}", exp));
        assert_eq!(format!("{:?}", res), format!("{:?}", exp));
    }
}
