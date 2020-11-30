#[derive(Debug, PartialEq, Eq)]
enum JsonValue {
    Null(),
    True(),
    False(),
    Object(),
    Array(),
    Int(i32),
}

struct JsonSegment<'a> {
    segment: &'a str,
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

impl<'a> JsonSegment<'a> {
    fn value(&self) -> Option<JsonValue> {
        for (i, c) in self.segment.char_indices() {
            return match c {
                '0' => Some(JsonValue::Int(0)),
                _ => None,
            };
        }

        None
    }

    fn parse_next(src: &str) -> (Option<JsonSegment>, usize) {
        for (i, c) in src.char_indices() {
            if is_space(c) {
                continue;
            }

            let segment = &src[i..];
            return (Some(JsonSegment { segment }), i);
        }

        (None, 0)
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
    assert_eq!(
        JsonValue::Int(1),
        JsonSegment::parse_next("1").0.unwrap().value().unwrap()
    );
}

fn main() {
    println!("Hello, world!");
}
