use super::parser::*;

pub struct JsonNode<'a> {
    parser: &'a Parser<'a>,
    index: usize,
}

#[derive(Debug, Clone)]
struct JsonNodeError {}
type JsonNodeResult<'a> = Result<JsonNode<'a>, JsonNodeError>;

impl<'a> JsonNode<'a> {
    pub fn new<'b>(parser: &'b Parser) -> JsonNode<'b> {
        JsonNode { parser, index: 0 }
    }

    fn slice(&self) -> &str {
        self.parser.get_slice(self.index)
    }

    fn get(&self, index: usize) -> JsonNodeResult {
        let token = &self.parser.tokens[self.index];
        match token {
            JsonToken::Value(_, value) => {
                match value {
                    JsonValue::ArrayOpen(_) => {
                        // let close = self.parser.tokens[*close_index];
                        let mut current = self.index + 1;
                        for _ in 0..index {
                            current = self.parser.next_sibling(current);
                        }
                        Ok(JsonNode {
                            parser: self.parser,
                            index: current,
                        })
                    }
                    _ => Err(JsonNodeError {}),
                }
            }
            _ => Err(JsonNodeError {}),
        }
    }

    fn key(&self, target: &str) -> JsonNodeResult {
        let token = &self.parser.tokens[self.index];
        match token {
            JsonToken::Value(_, value) => match value {
                JsonValue::ObjectOpen(close_index) => {
                    let mut current = self.index + 1;
                    while current < *close_index {
                        // key
                        let key_index = current;
                        let value_index = self.parser.next_sibling(key_index);

                        // value
                        let key = self.parser.get_slice(key_index);
                        if &key[1..key.len() - 1] == target {
                            return Ok(JsonNode {
                                parser: self.parser,
                                index: value_index,
                            });
                        }

                        current = self.parser.next_sibling(value_index);
                    }
                    // not found
                    Err(JsonNodeError {})
                }
                _ => Err(JsonNodeError {}),
            },
            _ => Err(JsonNodeError {}),
        }
    }
}

#[test]
fn slice_tests() {
    assert_eq!("1", JsonNode::new(&Parser::process(" 1")).slice());
    assert_eq!(
        r##""hoge""##,
        JsonNode::new(&Parser::process(r##" "hoge" "##)).slice()
    );
    assert_eq!(
        "[1, 2, 3]",
        JsonNode::new(&Parser::process(" [1, 2, 3]")).slice()
    );
    assert_eq!(
        r##"{"key": true}"##,
        JsonNode::new(&Parser::process(r##" {"key": true}"##)).slice()
    );
}

fn _node_tests<'a>() -> JsonNodeResult<'a> {
    {
        let parser = Parser::process("[1, 2, 3]");
        let array = JsonNode::new(&parser);
        assert_eq!("1", array.get(0)?.slice());
        assert_eq!("2", array.get(1)?.slice());
        assert_eq!("3", array.get(2)?.slice());
    }

    {
        let parser = Parser::process(r##"{ "key": true }"##);
        let obj = JsonNode::new(&parser);
        assert_eq!("true", obj.key("key")?.slice());
    }

    {
        let parser = Parser::process(r##"{ "key": {"key2": true }}"##);
        let obj = JsonNode::new(&parser);
        assert_eq!("true", obj.key("key")?.key("key2")?.slice());
    }

    Err(JsonNodeError {})
}
#[test]
fn node_tests() {
    _node_tests();
}
