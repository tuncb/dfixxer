use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;

struct Arguments {
    filename: String,
}

fn parse_args(args: Vec<String>) -> Arguments {
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    Arguments {
        filename: args[1].clone(),
    }
}

fn load_file(filename: &str) -> String {
    std::fs::read_to_string(filename).expect("Failed to read file")
}

fn parse_to_tree(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    parser.parse(source, None).expect("Failed to parse source")
}

fn find_kuses_nodes<'a>(tree: &'a Tree, _source: &str) -> Vec<Node<'a>> {
    fn traverse<'b>(node: Node<'b>, nodes: &mut Vec<Node<'b>>) {
        if node.kind() == "kUses" {
            nodes.push(node);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                traverse(child, nodes);
            }
        }
    }
    let mut nodes = Vec::new();
    traverse(tree.root_node(), &mut nodes);
    nodes
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_args(args);
    let source = load_file(&arguments.filename);
    let tree = parse_to_tree(&source);
    let kuses_nodes = find_kuses_nodes(&tree, &source);
    println!("Found {} kUses nodes.", kuses_nodes.len());
    for node in kuses_nodes {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        println!("kUses node: kind='{}', text='{}'", node.kind(), text);
    }
}
