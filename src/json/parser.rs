use std::fmt;

struct PeekIt<'a> {
    it: std::str::CharIndices<'a>,
    last: Option<(usize, char)>,
}

impl<'a> PeekIt<'a> {
    fn new(it: std::str::CharIndices) -> PeekIt {
        PeekIt { it, last: None }
    }

    fn next(&mut self) {
        self.last = self.it.next();
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.last
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JsonValue {
    Null(),
    True(),
    False(),
    Number(usize),     // byte len
    String(usize),     // byte len. include double quote
    ArrayOpen(usize),  // close index.
    ObjectOpen(usize), // close index.
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null() => write!(f, "null"),
            JsonValue::True() => write!(f, "true"),
            JsonValue::False() => write!(f, "false"),
            JsonValue::Number(len) => write!(f, "number[{}]", len),
            JsonValue::String(len) => write!(f, "string[{}]", len),
            JsonValue::ArrayOpen(_) => write!(f, "["),
            JsonValue::ObjectOpen(_) => write!(f, "{{"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JsonTokenData {
    Value(JsonValue),
    Comma(),
    Colon(),
    ArrayClose(usize),  // count
    ObjectClose(usize), // count
}

#[derive(Debug, Clone, Copy)]
pub struct JsonToken {
    start: usize,
    pub data: JsonTokenData,
}

impl fmt::Display for JsonTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonTokenData::Colon() => write!(f, ":"),
            JsonTokenData::Comma() => write!(f, ","),
            JsonTokenData::ArrayClose(_) => write!(f, "]"),
            JsonTokenData::ObjectClose(_) => write!(f, "}}"),
            JsonTokenData::Value(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone)]
enum ParseError {
    Eof(),
    Unknown(usize, char),
    Value(usize, JsonValue),
}
type ParseResult = Result<JsonToken, ParseError>;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Eof() => write!(f, "eof"),
            ParseError::Unknown(i, c) => write!(f, "'{}' at {} is unknown", c, i),
            ParseError::Value(i, value) => write!(f, "{} at {} is invalid", value, i),
        }
    }
}

pub struct JsonParser<'a> {
    pub src: &'a str,
    pub tokens: Vec<JsonToken>,
}

fn get_char(it: &mut PeekIt, expected: char) -> Result<usize, ParseError> {
    match it.peek() {
        Some((i, c)) => {
            if c == expected {
                Ok(i)
            } else {
                Err(ParseError::Unknown(i, c))
            }
        }
        None => Err(ParseError::Eof()),
    }
}

fn is_digit(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

impl JsonToken {
    fn get_null_token(it: &mut PeekIt, start: usize) -> ParseResult {
        get_char(it, 'u')?;
        it.next();
        get_char(it, 'l')?;
        it.next();
        get_char(it, 'l')?;
        it.next();
        Ok(JsonToken {
            start,
            data: JsonTokenData::Value(JsonValue::Null()),
        })
    }

    fn get_true_token(it: &mut PeekIt, start: usize) -> ParseResult {
        get_char(it, 'r')?;
        it.next();
        get_char(it, 'u')?;
        it.next();
        get_char(it, 'e')?;
        it.next();
        Ok(JsonToken {
            start,
            data: JsonTokenData::Value(JsonValue::True()),
        })
    }

    fn get_false_token(it: &mut PeekIt, start: usize) -> ParseResult {
        get_char(it, 'a')?;
        it.next();
        get_char(it, 'l')?;
        it.next();
        get_char(it, 's')?;
        it.next();
        get_char(it, 'e')?;
        it.next();
        Ok(JsonToken {
            start,
            data: JsonTokenData::Value(JsonValue::False()),
        })
    }

    fn get_number_token(it: &mut PeekIt, start: usize) -> ParseResult {
        let mut digit = start;
        let mut last = ' ';

        while let Some((i, c)) = it.peek() {
            if is_digit(c) {
                it.next();
                digit = i;
                continue;
            }
            last = c;
            break;
        }

        if last == '.' {
            it.next();
            while let Some((i, c)) = it.peek() {
                if is_digit(c) {
                    it.next();
                    digit = i;
                    continue;
                }
                last = c;
                break;
            }
        }

        if last == 'E' || last == 'e' {
            it.next();
            if let Some((i, c)) = it.peek() {
                if c == '+' || c == '-' {
                    it.next();
                    while let Some((i, c)) = it.peek() {
                        if is_digit(c) {
                            it.next();
                            digit = i;
                            continue;
                        }
                        break;
                    }
                } else {
                    return Err(ParseError::Unknown(i, c));
                }
            } else {
                return Err(ParseError::Eof());
            }
        }

        Ok(JsonToken {
            start,
            data: JsonTokenData::Value(JsonValue::Number(digit + 1 - start)),
        })
    }

    fn get_string_token(it: &mut PeekIt, start: usize) -> ParseResult {
        while let Some((i, c)) = it.peek() {
            it.next();
            if c == '"' {
                return Ok(JsonToken {
                    start,
                    data: JsonTokenData::Value(JsonValue::String(i + 1 - start)),
                });
            }
        }
        Err(ParseError::Eof())
    }
}

impl<'a> JsonParser<'a> {
    fn get_array_token(&mut self, it: &mut PeekIt) -> Result<usize, ParseError> {
        {
            // close or key
            let token = self.parse(it)?;
            match token.data {
                JsonTokenData::ArrayClose(_) => return Ok(0),
                JsonTokenData::ObjectClose(_) => return Err(ParseError::Unknown(token.start, '}')),
                JsonTokenData::Value(_) => (), // continue
                JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
            };
        }

        let mut count = 1;
        loop {
            // comma or close
            {
                let token = self.parse(it)?;
                match token.data {
                    JsonTokenData::ArrayClose(_) => return Ok(count),
                    JsonTokenData::ObjectClose(_) => {
                        return Err(ParseError::Unknown(token.start, '}'))
                    }
                    JsonTokenData::Value(value) => {
                        return Err(ParseError::Value(token.start, value))
                    }
                    JsonTokenData::Comma() => {
                        if count > 0 {
                            () // continue
                        } else {
                            return Err(ParseError::Unknown(token.start, ','));
                        }
                    }
                    JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
                };
            }

            // increment
            count += 1;

            // must value
            {
                let token = self.parse(it)?;
                match token.data {
                    JsonTokenData::ArrayClose(_) => {
                        return Err(ParseError::Unknown(token.start, ']'))
                    }
                    JsonTokenData::ObjectClose(_) => {
                        return Err(ParseError::Unknown(token.start, '}'))
                    }
                    JsonTokenData::Value(_) => (), // continue
                    JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                    JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
                };
            }
        }
    }

    fn colon_value(&mut self, it: &mut PeekIt) -> ParseResult {
        // :
        {
            let token = self.parse(it)?;
            match token.data {
                JsonTokenData::ArrayClose(_) => return Err(ParseError::Unknown(token.start, ']')),
                JsonTokenData::ObjectClose(_) => return Err(ParseError::Unknown(token.start, '}')),
                JsonTokenData::Value(value) => return Err(ParseError::Value(token.start, value)),
                JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                JsonTokenData::Colon() => (), // continue
            }
        }
        // value
        {
            let token = self.parse(it)?;
            match token.data {
                JsonTokenData::ArrayClose(_) => return Err(ParseError::Unknown(token.start, ']')),
                JsonTokenData::ObjectClose(_) => return Err(ParseError::Unknown(token.start, '}')),
                JsonTokenData::Value(_) => Ok(token),
                JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
            }
        }
    }

    fn get_object_token(&mut self, it: &mut PeekIt) -> Result<usize, ParseError> {
        {
            // close or key
            let token = self.parse(it)?;
            match token.data {
                JsonTokenData::ArrayClose(_) => return Err(ParseError::Unknown(token.start, ']')),
                JsonTokenData::ObjectClose(_) => return Ok(0),
                JsonTokenData::Value(_) => (), // continue
                JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
            };
            self.colon_value(it)?;
        }

        let mut count = 1;
        loop {
            {
                // comma or close
                let token = self.parse(it)?;
                match token.data {
                    JsonTokenData::ArrayClose(_) => {
                        return Err(ParseError::Unknown(token.start, ']'))
                    }
                    JsonTokenData::ObjectClose(_) => return Ok(count),
                    JsonTokenData::Value(value) => {
                        return Err(ParseError::Value(token.start, value))
                    }
                    JsonTokenData::Comma() => {
                        () // continue
                    }
                    JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
                };
            }
            // increment
            count += 1;
            // key
            {
                let token = self.parse(it)?;
                match token.data {
                    JsonTokenData::ArrayClose(_) => {
                        return Err(ParseError::Unknown(token.start, ']'))
                    }
                    JsonTokenData::ObjectClose(_) => {
                        return Err(ParseError::Unknown(token.start, '}'))
                    }
                    JsonTokenData::Value(_) => (),
                    JsonTokenData::Comma() => return Err(ParseError::Unknown(token.start, ',')),
                    JsonTokenData::Colon() => return Err(ParseError::Unknown(token.start, ':')),
                };
            }
            self.colon_value(it)?;
        }
    }

    fn parse(&mut self, it: &mut PeekIt) -> ParseResult {
        while let Some((i, c)) = it.peek() {
            it.next();
            if c.is_whitespace() {
                continue;
            }

            return match c {
                'n' => {
                    let token = JsonToken::get_null_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                't' => {
                    let token = JsonToken::get_true_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                'f' => {
                    let token = JsonToken::get_false_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                '0'..='9' | '-' => {
                    let token = JsonToken::get_number_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                '"' => {
                    let token = JsonToken::get_string_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                ',' => Ok(JsonToken {
                    start: i,
                    data: JsonTokenData::Comma(),
                }),
                '[' => {
                    // tmp open
                    let open_index = self.tokens.len();
                    self.tokens.push(JsonToken {
                        start: i,
                        data: JsonTokenData::Value(JsonValue::ArrayOpen(open_index + 1)),
                    });
                    let item_count = self.get_array_token(it)?;
                    let close_index = self.tokens.len() - 1;

                    // update open
                    let token = JsonToken {
                        start: i,
                        data: JsonTokenData::Value(JsonValue::ArrayOpen(close_index)),
                    };
                    self.tokens[open_index] = token;

                    // update close
                    self.tokens[close_index] = JsonToken {
                        start: self.tokens[close_index].start,
                        data: JsonTokenData::ArrayClose(item_count),
                    };

                    Ok(token)
                }
                ']' => {
                    let token = JsonToken {
                        start: i,
                        data: JsonTokenData::ArrayClose(0),
                    };
                    self.tokens.push(token);
                    Ok(token)
                }
                ':' => Ok(JsonToken {
                    start: i,
                    data: JsonTokenData::Colon(),
                }),
                '{' => {
                    // tmp open
                    let open_index = self.tokens.len();
                    self.tokens.push(JsonToken {
                        start: i,
                        data: JsonTokenData::Value(JsonValue::ObjectOpen(open_index + 1)),
                    });
                    let item_count = self.get_object_token(it)?;
                    let close_index = self.tokens.len() - 1;

                    // update open
                    let token = JsonToken {
                        start: i,
                        data: JsonTokenData::Value(JsonValue::ObjectOpen(close_index)),
                    };
                    self.tokens[open_index] = token;

                    // update close
                    self.tokens[close_index] = JsonToken {
                        start: self.tokens[close_index].start,
                        data: JsonTokenData::ObjectClose(item_count),
                    };

                    Ok(token)
                }
                '}' => {
                    let token = JsonToken {
                        start: i,
                        data: JsonTokenData::ObjectClose(0),
                    };
                    self.tokens.push(token);
                    Ok(token)
                }
                _ => Err(ParseError::Unknown(i, c)),
            };
        }
        Err(ParseError::Eof())
    }

    pub fn process(src: &str) -> JsonParser {
        let mut parser = JsonParser {
            src: src,
            tokens: Vec::new(),
        };

        let mut it = PeekIt::new(parser.src.char_indices());
        it.next();
        match parser.parse(&mut it) {
            Ok(_) => {
                return parser;
                // let end = parser.value_end(i, &value);
                // println!(
                //     r##""{}"[{}..{}] => {}"##,
                //     parser.src,
                //     i,
                //     end,
                //     &parser.src[i..end]
                // )
            }
            Err(error) => println!("{} => {}", parser.src, error),
            _ => print!("unknown"),
        }

        panic!()
    }

    pub fn next_sibling_index(&self, index: usize) -> usize {
        let token = self.tokens[index];
        match token.data {
            JsonTokenData::Value(value) => match value {
                JsonValue::ArrayOpen(close_index) => close_index + 1,
                JsonValue::ObjectOpen(close_index) => close_index + 1,
                _ => index + 1,
            },
            _ => panic!(),
        }
    }

    fn value_len(&self, value: JsonValue) -> usize {
        match value {
            JsonValue::Null() => 4,
            JsonValue::True() => 4,
            JsonValue::False() => 5,
            JsonValue::Number(n) => n,
            JsonValue::String(n) => n,
            _ => panic!(),
        }
    }

    pub fn get_slice(&self, index: usize) -> &str {
        let token = &self.tokens[index];
        let end = match token.data {
            JsonTokenData::Value(value) => match value {
                JsonValue::ArrayOpen(close_index) => {
                    let close = self.tokens[close_index];
                    close.start + 1
                }
                JsonValue::ObjectOpen(close_index) => {
                    let close = self.tokens[close_index];
                    close.start + 1
                }
                _ => token.start + self.value_len(value),
            },
            _ => token.start + 1,
        };

        &self.src[token.start..end]
    }

    pub fn get_int(&self, index: usize) -> Option<i64> {
        let token = &self.tokens[index];
        match token.data {
            JsonTokenData::Value(JsonValue::Number(len)) => {
                let segment = &self.src[token.start..token.start + len];
                if let Ok(value) = segment.parse::<i64>() {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_bool(&self, index: usize) -> Option<bool> {
        let token = &self.tokens[index];
        match token.data {
            JsonTokenData::Value(value) => match value {
                JsonValue::True() => Some(true),
                JsonValue::False() => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn get_string(&self, index: usize) -> Option<&str> {
        let token = &self.tokens[index];
        match token.data {
            JsonTokenData::Value(JsonValue::String(len)) => {
                Some(&self.src[token.start + 1..token.start + len - 1])
            }
            _ => None,
        }
    }
}
