use tree_sitter::Parser;
use tree_sitter_pascal::LANGUAGE;

fn main() {
    use std::env;
    use std::fs;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).expect("Failed to read file");

    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse(&source, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
}
