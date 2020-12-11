use super::parser::*;

pub struct JsonNode<'a> {
    parser: &'a JsonParser<'a>,
    index: usize,
}

impl<'a> std::fmt::Display for JsonNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parser.get_slice(self.index))
    }
}

#[derive(Debug, Clone)]
pub struct JsonNodeError {}
type JsonNodeResult<'a> = Result<JsonNode<'a>, JsonNodeError>;

pub struct JsonArrayIter<'a> {
    parser: &'a JsonParser<'a>,
    current: usize,
    end: usize,
}

pub struct JsonObjectIter<'a> {
    parser: &'a JsonParser<'a>,
    current: usize,
    end: usize,
}

impl<'a> Iterator for JsonObjectIter<'a> {
    type Item = (&'a str, JsonNode<'a>);

    fn next(&mut self) -> Option<(&'a str, JsonNode<'a>)> {
        if self.current == self.end {
            return None;
        }

        let key_index = self.current;
        let value_index = self.parser.next_sibling_index(key_index);
        self.current = self.parser.next_sibling_index(value_index);

        if let Some(key) = self.parser.get_string(key_index) {
            Some((key, JsonNode::from_index(self.parser, value_index)))
        } else {
            None
        }
    }
}

impl<'a> JsonNode<'a> {
    pub fn new<'b>(parser: &'b JsonParser) -> JsonNode<'b> {
        JsonNode { parser, index: 0 }
    }

    pub fn from_index<'b>(parser: &'b JsonParser, index: usize) -> JsonNode<'b> {
        JsonNode { parser, index }
    }

    pub fn token(&self) -> &JsonToken {
        &self.parser.tokens[self.index]
    }

    pub fn value(&self) -> JsonValue {
        let token = self.token();
        match token.data {
            JsonTokenData::Value(value) => value,
            _ => panic!(),
        }
    }

    pub fn slice(&self) -> &str {
        self.parser.get_slice(self.index)
    }

    pub fn get_int(&self) -> Option<i64> {
        self.parser.get_int(self.index)
    }

    pub fn get_string(&self) -> Option<&str> {
        self.parser.get_string(self.index)
    }

    pub fn get(&self, index: usize) -> JsonNodeResult {
        let token = self.token();
        match token.data {
            JsonTokenData::Value(value) => {
                match value {
                    JsonValue::ArrayOpen(_) => {
                        // let close = self.parser.tokens[*close_index];
                        let mut current = self.index + 1;
                        for _ in 0..index {
                            current = self.parser.next_sibling_index(current);
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

    pub fn array_len(&self) -> Option<usize> {
        None
    }

    pub fn key(&self, target: &str) -> JsonNodeResult {
        let token = self.token();
        match token.data {
            JsonTokenData::Value(value) => match value {
                JsonValue::ObjectOpen(close_index) => {
                    let mut current = self.index + 1;
                    while current < close_index {
                        // key
                        let key_index = current;
                        let value_index = self.parser.next_sibling_index(key_index);

                        // value
                        let key = self.parser.get_slice(key_index);
                        if &key[1..key.len() - 1] == target {
                            return Ok(JsonNode {
                                parser: self.parser,
                                index: value_index,
                            });
                        }

                        current = self.parser.next_sibling_index(value_index);
                    }
                    // not found
                    Err(JsonNodeError {})
                }
                _ => Err(JsonNodeError {}),
            },
            _ => Err(JsonNodeError {}),
        }
    }

    pub fn object_iter(&self) -> JsonObjectIter {
        let token = self.token();
        match token.data {
            JsonTokenData::Value(value) => match value {
                JsonValue::ObjectOpen(close_index) => {
                    return JsonObjectIter {
                        parser: self.parser,
                        current: self.index + 1,
                        end: close_index,
                    }
                }
                _ => (),
            },
            _ => (),
        }

        JsonObjectIter {
            parser: self.parser,
            current: self.index + 1,
            end: self.index + 1,
        }
    }

    pub fn object_len(&self) -> usize {
        0
    }
}

#[test]
fn slice_tests() {
    assert_eq!("1", JsonNode::new(&JsonParser::process(" 1")).slice());
    assert_eq!(
        r##""hoge""##,
        JsonNode::new(&JsonParser::process(r##" "hoge" "##)).slice()
    );
    assert_eq!(
        "[1, 2, 3]",
        JsonNode::new(&JsonParser::process(" [1, 2, 3]")).slice()
    );
    assert_eq!(
        r##"{"key": true}"##,
        JsonNode::new(&JsonParser::process(r##" {"key": true}"##)).slice()
    );
}

#[test]
fn node_tests<'a>() {
    {
        let parser = JsonParser::process("[1, 2, 3]");
        let array = JsonNode::new(&parser);

        assert_eq!(Some(3), array.array_len());
        assert_eq!(Some(1), array.get(0).unwrap().get_int());
        assert_eq!(Some(2), array.get(1).unwrap().get_int());
        assert_eq!(Some(3), array.get(2).unwrap().get_int());
    }

    {
        let parser = JsonParser::process(r##"{ "key": true }"##);
        let obj = JsonNode::new(&parser);
        assert_eq!("true", obj.key("key").unwrap().slice());
    }

    {
        let parser = JsonParser::process(r##"{ "key": {"key2": true }}"##);
        let obj = JsonNode::new(&parser);
        assert_eq!("true", obj.key("key").unwrap().key("key2").unwrap().slice());
    }
}
