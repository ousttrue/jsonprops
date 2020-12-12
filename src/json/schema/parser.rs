use super::super::node::{JsonNode, JsonNodeError, JsonNodeResult};
use super::super::parser::JsonParser;

pub struct JsonSchema {
    pub title: String,
    pub description: String,
    pub json_type: String,
}

impl JsonSchema {
    pub fn new() -> JsonSchema {
        JsonSchema {
            title: String::new(),
            description: String::new(),
            json_type: String::new(),
        }
    }

    // [ { "$ref": "glTFProperty.schema.json" } ]
    pub fn set_allof<'a>(&self, v: JsonNode<'a>) -> JsonNodeResult<'a> {
        if v.array_len() == Some(1) {
            if let Ok(node) = v.get(0) {
                if let Some(len) = node.object_len() {
                    if len == 1 {
                        let value = node.key("$ref")?;
                        if let Some(text) = value.get_string() {
                            println!("{}", text);
                            return Ok(value);
                        }
                    }
                }
            }
        }
        Err(JsonNodeError {})
    }
}

pub struct JsonSchemaParser {
    pub root: JsonSchema,
}

impl JsonSchemaParser {
    pub fn from_str(text: &str) -> JsonSchemaParser {
        let parser = JsonParser::process(&text);

        let root = JsonNode::new(&parser);

        // println!("ok");
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
                    schema.set_allof(v).unwrap();
                }
                "required" => {}
                "dependencies" => {}
                "properties" => {}
                _ => println!("{} => {}", k, v),
            }
        }

        JsonSchemaParser { root: schema }
    }
}
