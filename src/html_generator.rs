use super::json_diff::JsonV;
use std::collections::HashMap;

#[derive(Debug)]
enum Line {
    Same(String),
    DiffMissing(String),
    DiffPresent(String),
    NewLine,
    Text(String),
    Start,
}

#[derive(Debug)]
struct Node {
    previous: Option<Box<Node>>,
    content: Line,
}

pub fn generate(json: JsonV) -> String {
    let mut acc = Node {
        previous: None,
        content: Line::Start,
    };
    // acc = generate_rec(json, acc);

    println!("{:?}", acc);

    // acc.join("\n")
    "".to_string()
}

fn generate_rec(json: JsonV, acc: Node) -> Node {
    match json {
        JsonV::Null(st) => {
            Node { previous: Some(Box::new(acc)), content: Line::Same("null".to_string())}
        }
        JsonV::String(s, st) => {
            Node { previous: Some(Box::new(acc)), content: Line::Same(s)}
        }
        JsonV::Bool(b, st) => {
            let s2 = if b { "true" } else { "false" };
            Node { previous: Some(Box::new(acc)), content: Line::Same(s2.to_string())}
        }
        JsonV::Number(n, st) => {
            Node { previous: Some(Box::new(acc)), content: Line::Same(n.to_string())}
        }
        JsonV::Array(v, st) => {
            // acc.push_str("\n");
            Node { previous: Some(Box::new(acc)), content: Line::Same("".to_string())}
        }
        JsonV::Object(h, st) => {
            // let t = acc.pop().unwrap_or("".to_string());
            // acc.push(t + "{");
            // for (k, v) in h {
            //     acc.push(format!("\"{}\": ", k));
            //     acc = generate_rec(v, acc);
            // }
            // acc.push("},".to_string());
            Node { previous: Some(Box::new(acc)), content: Line::Same("".to_string())}
        }
    }
}

// fn add_new(v: Vec<Line>, s: String) -> Vec<Line> {}

// fn add_to_last(v: Vec<Line>, s: String) -> Vec<Line> {
//     let t = v.pop();

//     v.push(t);
// }

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
        h.insert("item3".to_string(), JsonV::Object(h2, Vec::new()));

        let json = JsonV::Object(h, Vec::new());
        let res = generate(json);
        println!("{}", res);
    }
}
