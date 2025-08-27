use tree_sitter::Parser;
use tree_sitter_pascal::LANGUAGE;

fn main() {
    let mut parser = Parser::new();

    parser.set_language(&LANGUAGE.into()).unwrap();

    let tree = parser.parse("unit Test;", None).unwrap();

    println!("{}", tree.root_node().to_sexp());
}
