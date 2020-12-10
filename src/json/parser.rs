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
    String(usize),     // byte len
    ArrayOpen(usize),  // close index
    ObjectOpen(usize), // close index
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
pub enum JsonToken {
    Value(usize, JsonValue),
    Comma(usize),
    Colon(usize),
    ArrayClose(usize),
    ObjectClose(usize),
}

impl fmt::Display for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonToken::Colon(_) => write!(f, ":"),
            JsonToken::Comma(_) => write!(f, ","),
            JsonToken::ArrayClose(_) => write!(f, "]"),
            JsonToken::ObjectClose(_) => write!(f, "}}"),
            JsonToken::Value(_, value) => write!(f, "{}", value),
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

pub struct Parser<'a> {
    src: &'a str,
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

fn get_null_token(it: &mut PeekIt, start: usize) -> ParseResult {
    get_char(it, 'u')?;
    it.next();
    get_char(it, 'l')?;
    it.next();
    get_char(it, 'l')?;
    it.next();
    Ok(JsonToken::Value(start, JsonValue::Null()))
}

fn get_true_token(it: &mut PeekIt, start: usize) -> ParseResult {
    get_char(it, 'r')?;
    it.next();
    get_char(it, 'u')?;
    it.next();
    get_char(it, 'e')?;
    it.next();
    Ok(JsonToken::Value(start, JsonValue::True()))
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
    Ok(JsonToken::Value(start, JsonValue::False()))
}

fn is_digit(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
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

    Ok(JsonToken::Value(
        start,
        JsonValue::Number(digit + 1 - start),
    ))
}

fn get_string_token(it: &mut PeekIt, start: usize) -> ParseResult {
    while let Some((i, c)) = it.peek() {
        it.next();
        if c == '"' {
            return Ok(JsonToken::Value(start, JsonValue::String(i + 1 - start)));
        }
    }
    Err(ParseError::Eof())
}

impl<'a> Parser<'a> {
    fn get_array_token(&mut self, it: &mut PeekIt) -> ParseResult {
        {
            // value or close
            match self.parse(it)? {
                token @ JsonToken::ArrayClose(_) => return Ok(token),
                JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
                JsonToken::Value(_, _) => (), // continue
                JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
        }
        loop {
            // comma or close
            match self.parse(it)? {
                token @ JsonToken::ArrayClose(_) => return Ok(token),
                JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
                JsonToken::Value(i, value) => return Err(ParseError::Value(i, value)),
                JsonToken::Comma(_) => (), // continue
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
            // must value
            match self.parse(it)? {
                JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
                JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
                JsonToken::Value(_, _) => (), // continue
                JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
        }
    }

    fn colon_value(&mut self, it: &mut PeekIt) -> ParseResult {
        // :
        match self.parse(it)? {
            JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
            JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
            JsonToken::Value(i, value) => return Err(ParseError::Value(i, value)),
            JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
            JsonToken::Colon(_) => (), // continue
        }
        // value
        match self.parse(it)? {
            JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
            JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
            JsonToken::Value(i, value) => Ok(JsonToken::Value(i, value)),
            JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
            JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
        }
    }

    fn get_object_token(&mut self, it: &mut PeekIt) -> ParseResult {
        {
            // key or close
            match self.parse(it)? {
                JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
                token @ JsonToken::ObjectClose(_) => return Ok(token),
                JsonToken::Value(_, _) => (), // continue
                JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
            self.colon_value(it)?;
        }

        loop {
            // camma or close
            match self.parse(it)? {
                JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
                token @ JsonToken::ObjectClose(_) => return Ok(token),
                JsonToken::Value(i, value) => return Err(ParseError::Value(i, value)),
                JsonToken::Comma(_) => (), // continue
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
            // key
            match self.parse(it)? {
                JsonToken::ArrayClose(i) => return Err(ParseError::Unknown(i, ']')),
                JsonToken::ObjectClose(i) => return Err(ParseError::Unknown(i, '}')),
                JsonToken::Value(_, _) => (),
                JsonToken::Comma(i) => return Err(ParseError::Unknown(i, ',')),
                JsonToken::Colon(i) => return Err(ParseError::Unknown(i, ':')),
            };
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
                    let token = get_null_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                't' => {
                    let token = get_true_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                'f' => {
                    let token = get_false_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                '0'..='9' | '-' => {
                    let token = get_number_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                '"' => {
                    let token = get_string_token(it, i)?;
                    self.tokens.push(token);
                    Ok(token)
                }
                ',' => Ok(JsonToken::Comma(i)),
                '[' => {
                    let index = self.tokens.len();
                    self.tokens
                        .push(JsonToken::Value(i, JsonValue::ArrayOpen(index + 1)));
                    self.get_array_token(it)?;

                    // update close
                    let end_index = self.tokens.len() - 1;
                    let token = JsonToken::Value(i, JsonValue::ArrayOpen(end_index));
                    self.tokens[index] = token;
                    Ok(token)
                }
                ']' => {
                    let token = JsonToken::ArrayClose(i);
                    self.tokens.push(token);
                    Ok(token)
                }
                ':' => Ok(JsonToken::Colon(i)),
                '{' => {
                    let index = self.tokens.len();
                    self.tokens
                        .push(JsonToken::Value(i, JsonValue::ObjectOpen(index + 1)));
                    self.get_object_token(it)?;

                    // update close
                    let end_index = self.tokens.len() - 1;
                    let token = JsonToken::Value(i, JsonValue::ObjectOpen(end_index));
                    self.tokens[index] = token;
                    Ok(token)
                }
                '}' => {
                    let token = JsonToken::ObjectClose(i);
                    self.tokens.push(token);
                    Ok(token)
                }
                _ => Err(ParseError::Unknown(i, c)),
            };
        }
        Err(ParseError::Eof())
    }

    pub fn process(src: &str) -> Parser {
        let mut parser = Parser {
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

    fn value_end(&self, i: usize, value: &JsonValue) -> usize {
        match value {
            JsonValue::Null() => i + 4,
            JsonValue::True() => i + 4,
            JsonValue::False() => i + 5,
            JsonValue::Number(n) => i + *n,
            JsonValue::String(n) => i + *n,
            JsonValue::ArrayOpen(index) => {
                let close = &self.tokens[*index];
                self.end(close)
            }
            JsonValue::ObjectOpen(index) => {
                let close = &self.tokens[*index];
                self.end(close)
            }
        }
    }

    fn end(&self, token: &JsonToken) -> usize {
        match token {
            JsonToken::Value(i, value) => self.value_end(*i, value),
            JsonToken::Comma(i) => *i + 1,
            JsonToken::Colon(i) => *i + 1,
            JsonToken::ArrayClose(i) => *i + 1,
            JsonToken::ObjectClose(i) => *i + 1,
        }
    }

    pub fn next_sibling(&self, index: usize) -> usize {
        let token = self.tokens[index];
        match token {
            JsonToken::Value(_, value) => match value {
                JsonValue::ArrayOpen(close_index) => close_index + 1,
                JsonValue::ObjectOpen(close_index) => close_index + 1,
                _ => index + 1,
            },
            _ => panic!(),
        }
    }

    pub fn get_slice(&self, index: usize) -> &str {
        let token = &self.tokens[index];
        match token {
            JsonToken::Value(i, _) => &self.src[*i..self.end(token)],
            _ => panic!(),
        }
    }
}
