use json::schema::parser::JsonSchemaParser;

mod json;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: jsonprop.exe {{input.json}}");
        return;
    }

    let src = std::fs::read_to_string(&args[1]).unwrap();

    let parser = JsonSchemaParser::parse(&src);

    print!("done");
}
