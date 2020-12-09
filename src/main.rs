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

#[derive(Debug, Clone)]
enum JsonValue {
    Null(),
    True(),
    False(),
    Number(usize),
    String(usize),
    Array(usize),
    Object(usize),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null() => write!(f, "null"),
            JsonValue::True() => write!(f, "true"),
            JsonValue::False() => write!(f, "false"),
            JsonValue::Number(len) => write!(f, "number[{}]", len),
            JsonValue::String(len) => write!(f, "string[{}]", len),
            JsonValue::Array(len) => write!(f, "array[{}]", len),
            JsonValue::Object(len) => write!(f, "object[{}]", len),
        }
    }
}

enum JsonToken {
    Value(usize, JsonValue),
    Comma(usize),
    Colon(usize),
    CloseArray(usize),
    CloseObject(usize),
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

fn get_array_token(it: &mut PeekIt, start: usize) -> ParseResult {
    {
        // value or close
        match parse(it)? {
            JsonToken::Value(_, _) => (),
            JsonToken::CloseArray(e) => {
                return Ok(JsonToken::Value(start, JsonValue::Array(e + 1 - start)))
            }
            //
            JsonToken::Comma(e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::CloseObject(e) => return Err(ParseError::Unknown(e, '}')),
        };
    }

    loop {
        // comma or close
        match parse(it)? {
            JsonToken::Comma(_) => (),
            JsonToken::CloseArray(e) => {
                return Ok(JsonToken::Value(start, JsonValue::Array(e + 1 - start)))
            }
            //
            JsonToken::Value(e, v) => return Err(ParseError::Value(e, v)),
            JsonToken::CloseObject(e) => return Err(ParseError::Unknown(e, '}')),
            JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ':')),
        };

        // must value
        match parse(it)? {
            JsonToken::Value(_, _) => (),
            //
            JsonToken::CloseArray(e) => return Err(ParseError::Unknown(e, ']')),
            JsonToken::Comma(e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::CloseObject(e) => return Err(ParseError::Unknown(e, '}')),
            JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ':')),
        };
    }
}

fn colon_value(it: &mut PeekIt) -> ParseResult {
    match parse(it)? {
        JsonToken::Colon(_) => (),
        //
        JsonToken::CloseObject(e) => return Err(ParseError::Unknown(e, '}')),
        JsonToken::Comma(e) => return Err(ParseError::Unknown(e, ',')),
        JsonToken::Value(e, value) => return Err(ParseError::Value(e, value)),
        JsonToken::CloseArray(e) => return Err(ParseError::Unknown(e, ']')),
    }

    match parse(it)? {
        JsonToken::Value(e, value) => Ok(JsonToken::Value(e, value)),
        //
        JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ':')),
        JsonToken::CloseObject(e) => return Err(ParseError::Unknown(e, '}')),
        JsonToken::Comma(e) => return Err(ParseError::Unknown(e, ',')),
        JsonToken::CloseArray(e) => return Err(ParseError::Unknown(e, ']')),
    }
}

fn get_object_token(it: &mut PeekIt, start: usize) -> ParseResult {
    {
        // key or close
        match parse(it)? {
            JsonToken::Value(_, _) => (),
            JsonToken::CloseObject(e) => {
                return Ok(JsonToken::Value(start, JsonValue::Object(e + 1 - start)))
            }
            //
            JsonToken::Comma(e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ':')),
            JsonToken::CloseArray(e) => return Err(ParseError::Unknown(e, ']')),
        };
        colon_value(it)?;
    }

    loop {
        // camma or close
        match parse(it)? {
            JsonToken::Comma(_) => (),
            JsonToken::CloseObject(e) => {
                return Ok(JsonToken::Value(start, JsonValue::Object(e + 1 - start)))
            }
            //
            JsonToken::Value(e, value) => return Err(ParseError::Value(e, value)),
            JsonToken::Colon(e) => return Err(ParseError::Unknown(e, ':')),
            JsonToken::CloseArray(e) => return Err(ParseError::Unknown(e, '}')),
        };
        colon_value(it)?;
    }
}

fn parse(it: &mut PeekIt) -> ParseResult {
    while let Some((i, c)) = it.peek() {
        it.next();
        if c.is_whitespace() {
            continue;
        }

        return match c {
            'n' => get_null_token(it, i),               // null
            't' => get_true_token(it, i),               // true
            'f' => get_false_token(it, i),              // false
            '0'..='9' | '-' => get_number_token(it, i), // number
            '"' => get_string_token(it, i),             // string
            ',' => Ok(JsonToken::Comma(i)),             // comma
            ']' => Ok(JsonToken::CloseArray(i)),        // close array
            '[' => get_array_token(it, i),              // open array
            ':' => Ok(JsonToken::Colon(i)),             // colon
            '}' => Ok(JsonToken::CloseObject(i)),       // close object
            '{' => get_object_token(it, i),             // open object
            _ => Err(ParseError::Unknown(i, c)),
        };
    }
    Err(ParseError::Eof())
}

fn process(src: &str) {
    let mut it = PeekIt::new(src.char_indices());
    it.next();

    match parse(&mut it) {
        Ok(JsonToken::Value(e, value)) => println!("number: '{}[{}..] => '{}'", src, e, value),
        Err(error) => println!("{} => {}", src, error),
        _ => print!("unknown"),
    }
}

fn main() {
    process(" 1");
    process("2 ");
    process(r##" "hoge" "##);
    process("[1, 2, 3]");
    process(r##" {"key": "value"} "##);
    process(r##" {"key": {"key2": 1}} "##);

    println!();
}
