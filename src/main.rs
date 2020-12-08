#[derive(Debug, Clone)]
enum ParseError {
    Eof(),
    Position(usize),
}

fn get_null_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    match it.next() {
        Some((_, 'u')) => match it.next() {
            Some((_, 'l')) => match it.next() {
                Some((i, 'l')) => Ok((start, i)),
                Some((i, _)) => Err(ParseError::Position(i)),
                None => Err(ParseError::Eof()),
            },
            Some((i, _)) => Err(ParseError::Position(i)),
            None => Err(ParseError::Eof()),
        },
        Some((i, _)) => Err(ParseError::Position(i)),
        None => Err(ParseError::Eof()),
    }
}

fn get_true_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    match it.next() {
        Some((_, 'r')) => match it.next() {
            Some((_, 'u')) => match it.next() {
                Some((i, 'e')) => Ok((start, i)),
                Some((i, _)) => Err(ParseError::Position(i)),
                None => Err(ParseError::Eof()),
            },
            Some((i, _)) => Err(ParseError::Position(i)),
            None => Err(ParseError::Eof()),
        },
        Some((i, _)) => Err(ParseError::Position(i)),
        None => Err(ParseError::Eof()),
    }
}

fn get_false_token(
    it: &mut std::str::CharIndices,
    start: usize,
) -> Result<(usize, usize), ParseError> {
    match it.next() {
        Some((_, 'a')) => match it.next() {
            Some((_, 'l')) => match it.next() {
                Some((_, 's')) => match it.next() {
                    Some((i, 'e')) => Ok((start, i)),
                    Some((i, _)) => Err(ParseError::Position(i)),
                    None => Err(ParseError::Eof()),
                },
                Some((i, _)) => Err(ParseError::Position(i)),
                None => Err(ParseError::Eof()),
            },
            Some((i, _)) => Err(ParseError::Position(i)),
            None => Err(ParseError::Eof()),
        },
        Some((i, _)) => Err(ParseError::Position(i)),
        None => Err(ParseError::Eof()),
    }
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
