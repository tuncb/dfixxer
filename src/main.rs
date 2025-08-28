use std::fmt;
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;

#[derive(Debug)]
enum DfixxerError {
    InvalidArgs(String),
    IoError(std::io::Error),
    ParseError(String),
}

impl fmt::Display for DfixxerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DfixxerError::InvalidArgs(msg) => write!(f, "{}", msg),
            DfixxerError::IoError(err) => write!(f, "Failed to read file: {}", err),
            DfixxerError::ParseError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DfixxerError {}

impl From<std::io::Error> for DfixxerError {
    fn from(err: std::io::Error) -> Self {
        DfixxerError::IoError(err)
    }
}

struct Arguments {
    filename: String,
}

fn parse_args(args: Vec<String>) -> Result<Arguments, DfixxerError> {
    if args.len() < 2 {
        return Err(DfixxerError::InvalidArgs(format!(
            "Usage: {} <filename>",
            args[0]
        )));
    }
    Ok(Arguments {
        filename: args[1].clone(),
    })
}

fn load_file(filename: &str) -> Result<String, DfixxerError> {
    Ok(std::fs::read_to_string(filename)?)
}

fn parse_to_tree(source: &str) -> Result<Tree, DfixxerError> {
    let mut parser = Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .map_err(|_| DfixxerError::ParseError("Failed to set language".to_string()))?;
    parser
        .parse(source, None)
        .ok_or_else(|| DfixxerError::ParseError("Failed to parse source".to_string()))
}

fn find_kuses_nodes<'a>(tree: &'a Tree, _source: &str) -> Result<Vec<Node<'a>>, DfixxerError> {
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
    Ok(nodes)
}

fn run() -> Result<(), DfixxerError> {
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_args(args)?;
    let source = load_file(&arguments.filename)?;
    let tree = parse_to_tree(&source)?;
    let kuses_nodes = find_kuses_nodes(&tree, &source)?;

    println!("Found {} kUses nodes.", kuses_nodes.len());
    for node in kuses_nodes {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        println!("kUses node: kind='{}', text='{}'", node.kind(), text);
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
