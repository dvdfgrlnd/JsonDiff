use serde_json::Result;

mod edit_distance;
mod html_generator;
mod json_diff;

fn main() -> Result<()> {
    println!("Hello, world!");

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
    let json = json_diff::diff(r, r2)?;

    // println!("{:?}", json);
    let html_str = html_generator::generate(json);
    println!("{}", html_str);

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
}
