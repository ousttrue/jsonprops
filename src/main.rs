enum JsonToken {
    Value(usize, usize),
    Comma(usize, usize),
    CloseArray(usize, usize),
    CloseObject(usize, usize),
}

#[derive(Debug, Clone)]
enum ParseError {
    Eof(),
    Unknown(usize, char),
}
type ParseResult = Result<JsonToken, ParseError>;

use std::fmt;
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Eof() => write!(f, "eof"),
            ParseError::Unknown(i, c) => write!(f, "'{}' at {} is unknown", c, i),
        }
    }
}

fn get_char(it: &mut std::str::CharIndices, expected: char) -> Result<usize, ParseError> {
    match it.next() {
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

fn get_null_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    let _ = get_char(it, 'u')?;
    let _ = get_char(it, 'l')?;
    let end = get_char(it, 'l')?;
    Ok(JsonToken::Value(start, end))
}

fn get_true_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    let _ = get_char(it, 'r')?;
    let _ = get_char(it, 'u')?;
    let end = get_char(it, 'e')?;
    Ok(JsonToken::Value(start, end))
}

fn get_false_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    let _ = get_char(it, 'a')?;
    let _ = get_char(it, 'l')?;
    let _ = get_char(it, 's')?;
    let end = get_char(it, 'e')?;
    Ok(JsonToken::Value(start, end))
}

fn is_digit(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

fn get_number_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    let mut digit = start;
    let mut last = ' ';

    while let Some((i, c)) = it.next() {
        if is_digit(c) {
            digit = i;
            continue;
        }
        last = c;
        break;
    }

    if last == '.' {
        while let Some((i, c)) = it.next() {
            if is_digit(c) {
                digit = i;
                continue;
            }
            last = c;
            break;
        }
    }

    if last == 'E' || last == 'e' {
        if let Some((i, c)) = it.next() {
            if c == '+' || c == '-' {
                while let Some((i, c)) = it.next() {
                    if is_digit(c) {
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

    Ok(JsonToken::Value(start, digit))
}

fn get_string_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    while let Some((i, c)) = it.next() {
        if c == '"' {
            return Ok(JsonToken::Value(start, i));
        }
    }
    Err(ParseError::Eof())
}

fn is_close_array(token: JsonToken) -> (bool, usize) {
    match token {
        JsonToken::CloseArray(_, e) => (true, e),
        _ => (false, 0),
    }
}

fn get_array_token(it: &mut std::str::CharIndices, start: usize) -> ParseResult {
    {
        // value or close
        match parse(it)? {
            JsonToken::Value(_, _) => (),
            JsonToken::CloseArray(_, e) => return Ok(JsonToken::Value(start, e)),
            //
            JsonToken::Comma(_, e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::CloseObject(_, e) => return Err(ParseError::Unknown(e, '}')),
        };
    }

    loop {
        // camma or close
        match parse(it)? {
            JsonToken::Comma(_, _) => (),
            JsonToken::CloseArray(_, e) => return Ok(JsonToken::Value(start, e)),
            //
            JsonToken::Value(_, e) => return Err(ParseError::Unknown(e, ' ')),
            JsonToken::CloseObject(_, e) => return Err(ParseError::Unknown(e, '}')),
        };

        // must value
        match parse(it)? {
            JsonToken::Value(_, _) => (),
            //
            JsonToken::CloseArray(_, e) => return Err(ParseError::Unknown(e, ']')),
            JsonToken::Comma(_, e) => return Err(ParseError::Unknown(e, ',')),
            JsonToken::CloseObject(_, e) => return Err(ParseError::Unknown(e, '}')),
        };
    }
}

fn parse(it: &mut std::str::CharIndices) -> ParseResult {
    while let Some((i, c)) = it.next() {
        if c.is_whitespace() {
            continue;
        }

        return match c {
            'n' => get_null_token(it, i),               // null
            't' => get_true_token(it, i),               // true
            'f' => get_false_token(it, i),              // false
            '0'..='9' | '-' => get_number_token(it, i), // number
            '"' => get_string_token(it, i),             // string
            ',' => Ok(JsonToken::Comma(i, i)),          // comma
            ']' => Ok(JsonToken::CloseArray(i, i)),     // close array
            '[' => get_array_token(it, i),              // open array
            _ => Err(ParseError::Unknown(i, c)),
        };
    }
    Err(ParseError::Eof())
}

fn process(src: &str) {
    let mut it = src.char_indices();

    match parse(&mut it) {
        Ok(JsonToken::Value(s, e)) => println!(
            "number: '{}'[{}..{}] => '{}'",
            src,
            s,
            e + 1,
            &src[s..e + 1]
        ),
        Err(error) => println!("{} => {}", src, error),
        _ => print!("unknown"),
    }
}

fn main() {
    process(" 1");
    process("2 ");
    process(r##" "hoge" "##);
    process("[1, 2, 3]");
    println!();
}
