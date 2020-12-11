mod json;

struct JsonSchema {
    title: String,
    description: String,
    json_type: String,
}

impl JsonSchema {
    fn new() -> JsonSchema {
        JsonSchema {
            title: String::new(),
            description: String::new(),
            json_type: String::new(),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: jsonprop.exe {{input.json}}");
        return;
    }

    let text = std::fs::read_to_string(&args[1]).unwrap();

    let parser = json::parser::JsonParser::process(&text);

    let root = json::node::JsonNode::new(&parser);
    println!("ok");

    let mut schema = JsonSchema::new();
    for (k, v) in root.object_iter() {
        match k {
            "$schema" => {}
            "title" => {
                if let Some(title) = v.get_string() {
                    schema.title = title.to_string();
                }
            }
            "description" => {
                if let Some(description) = v.get_string() {
                    schema.description = description.to_string();
                }
            }
            "type" => {
                if let Some(json_type) = v.get_string() {
                    schema.json_type = json_type.to_string();
                }
            }
            "allOf" => {
                // [ { "$ref": "glTFProperty.schema.json" } ]
                if v.array_len() != Some(1) {
                    panic!();
                }
                if let Ok(kv) = v.get(0) {
                } else {
                    panic!();
                }
            }
            "required" => {}
            "dependencies" => {}
            "properties" => {}
            _ => println!("{} => {}", k, v),
        }
    }

    print!("done");
}
