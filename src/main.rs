mod json;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: jsonprop.exe {{input.json}}");
        return;
    }

    let text = std::fs::read_to_string(&args[1]).unwrap();

    let parser = json::parser::Parser::process(&text);

    let root = json::node::JsonNode::new(&parser);
    println!("ok");
}
