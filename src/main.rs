use tree_sitter::Parser;
use tree_sitter_pascal::LANGUAGE;

fn print_kuses_nodes(tree: &tree_sitter::Tree, source: &str) {
    fn print_kuses_siblings(start_node: tree_sitter::Node, source_bytes: &[u8]) {
        let mut node = Some(start_node);
        let mut printing = true;
        while let Some(n) = node {
            let kind = n.kind();
            let text = n.utf8_text(source_bytes).unwrap_or("");
            if kind == ";" {
                println!("End node ';' reached. Stopping sibling print.");
                break;
            }
            if n.is_error() {
                println!("ERROR: Node kind='{}', text='{}'", kind, text);
                printing = false;
            }
            if printing {
                println!("Sibling: kind='{}', text='{}'", kind, text);
            }
            node = n.next_sibling();
            if !printing && kind != "kUses" {
                // Wait for next kUses to resume printing
                while let Some(next) = node {
                    if next.kind() == "kUses" {
                        println!("Resuming at kUses node.");
                        printing = true;
                        node = Some(next);
                        break;
                    }
                    node = next.next_sibling();
                }
            }
        }
    }

    // Traverse tree to find kUses nodes at any depth
    fn traverse(node: tree_sitter::Node, source_bytes: &[u8]) {
        if node.kind() == "kUses" {
            println!(
                "Found kUses node: kind='{}', text='{}'",
                node.kind(),
                node.utf8_text(source_bytes).unwrap_or("")
            );
            print_kuses_siblings(node, source_bytes);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                traverse(child, source_bytes);
            }
        }
    }
    traverse(tree.root_node(), source.as_bytes());
}

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
    // ...existing code...
    print_kuses_nodes(&tree, &source);
}
