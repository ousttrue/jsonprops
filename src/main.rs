#[derive(Debug, PartialEq, Eq)]
enum JsonValue {
    Null(),
    True(),
    False(),
    Object(),
    Array(),
    Int(i32),
}

struct JsonParser<'a> {
    src: &'a str,
    segments: Vec<JsonSegment<'a>>,
}

struct JsonSegment<'a> {
    index: usize,
    offset: usize,
    bytes: usize,
    data: Box<JsonParser<'a>>,
}

fn is_space(c: char) -> bool {
    match (c) {
        ' ' => true,
        '\r' => true,
        '\n' => true,
        '\t' => true,
        _ => false,
    }
}

fn skip_space(src: &str) -> &str {
    for (i, c) in src.chars().enumerate() {
        if !is_space(c) {
            return &src[i..];
        }
    }

    src
}

impl<'a> JsonParser<'a> {
    // fn value(&self) -> Option<JsonValue> {
    //     for (i, c) in self.segment.char_indices() {
    //         return match c {
    //             '0' => Some(JsonValue::Int(0)),
    //             _ => None,
    //         };
    //     }

    //     None
    // }

    ///
    /// parse and return root value
    /// 
    fn parse(src: &'a str) -> Option<JsonSegment<'a>> {
        let parser = JsonParser {
            src,
            segments: Vec::new(),
        };

        for (i, c) in (&parser.src).char_indices() {
            if is_space(c) {
                continue;
            }
        }
        None
    }
}

#[test]
fn test_util() {
    assert_eq!(true, is_space(' '));
    assert_eq!(true, is_space('\r'));
    assert_eq!(false, is_space('🍎'));
    assert_eq!(false, is_space('字'));

    assert_eq!("1", skip_space(" 1"));
    assert_eq!("1 ", skip_space(" 1 "));
}

#[test]
fn parse_int() {
    // assert_eq!(
    //     JsonValue::Int(1),
    //     JsonParser::parse("1").value()
    // );
}

fn main() {
    println!("Hello, world!");
}
