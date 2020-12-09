#[derive(Debug, Clone)]
enum ParseError {
    Eof(),
    Position(usize),
}
type ParseResult = Result<(usize, usize), ParseError>;

fn get_char(it: &mut std::str::CharIndices, expected: char) -> ParseResult {
    match it.next() {
        Some((i, c)) => {
            if c == expected {
                Ok((i, i))
            } else {
                Err(ParseError::Position(i))
            }
        }
        None => Err(ParseError::Eof()),
    }
}

fn get_null_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    let _ = get_char(it, 'u')?;
    let _ = get_char(it, 'l')?;
    let (_, end) = get_char(it, 'l')?;
    Ok((start, end))
}

fn get_true_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    let _ = get_char(it, 'r')?;
    let _ = get_char(it, 'u')?;
    let (_, end) = get_char(it, 'e')?;
    Ok((start, end))
}

fn get_false_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    let _ = get_char(it, 'a')?;
    let _ = get_char(it, 'l')?;
    let _ = get_char(it, 's')?;
    let (_, end) = get_char(it, 'e')?;
    Ok((start, end))
}

fn get_number_token(it: &mut std::str::CharIndices, start: usize) -> (usize, usize) {
    let mut current = start;
    while let Some((i, c)) = it.next() {
        match c {
            '0'..='9' => current = i,
            _ => break,
        }
    }
    (start, current)
}

fn process(src: &str) {
    println!("#### '{}' ####", src);
    let mut it = src.char_indices();

    while let Some((i, c)) = it.next() {
        println!("{}: {}", i, c);
        if c.is_whitespace() {
            continue;
        }

        let (s, e) = match c {
            'n' => get_null_token(&mut it, i).unwrap(),
            't' => get_true_token(&mut it, i).unwrap(),
            'f' => get_false_token(&mut it, i).unwrap(),
            '0'..='9' => get_number_token(&mut it, i),
            _ => panic!(),
        };

        println!("number: {}..{} => '{}'", s, e, &src[s..e + 1]);
    }

    println!();
}

fn main() {
    process(" 1");
    process("2 ");
    process(" 345");
    process(" null 3 null");
    process(" true false 123 null");
}
