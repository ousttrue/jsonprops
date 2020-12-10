mod json;

fn main() {
    json::parser::Parser::process("2 ");
    json::parser::Parser::process(r##" "hoge" "##);
    json::parser::Parser::process("[1, 2, 3]");
    json::parser::Parser::process(r##" {"key": "value"} "##);
    let parser = json::parser::Parser::process(r##" {"key": {"key2": 1}} "##);

    println!();
}
