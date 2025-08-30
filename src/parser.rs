use crate::dfixxer_error::DFixxerError;
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;

/// Enum representing the kind of parsed node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Uses,
    Semicolon,
    Module,
    Comment,
    Preprocessor,
}

/// Struct to store parsed text block information independent of tree-sitter types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedNode {
    /// Kind of the parsed node
    pub kind: Kind,
    /// Start byte position in the original text
    pub start_byte: usize,
    /// End byte position in the original text
    pub end_byte: usize,
    /// Start row (0-based)
    pub start_row: usize,
    /// Start column (0-based)
    pub start_column: usize,
    /// End row (0-based)
    pub end_row: usize,
    /// End column (0-based)
    pub end_column: usize,
}

/// Struct representing a uses section in the parsed text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsesSection {
    pub uses: ParsedNode,
    pub siblings: Vec<ParsedNode>,
    pub semicolon: ParsedNode,
}

/// Struct representing the result of parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub uses_sections: Vec<UsesSection>,
}

fn parse_to_tree(source: &str) -> Result<Tree, DFixxerError> {
    let mut parser = Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .map_err(|_| DFixxerError::ParseError("Failed to set language".to_string()))?;
    parser
        .parse(source, None)
        .ok_or_else(|| DFixxerError::ParseError("Failed to parse source".to_string()))
}

/// Convert a tree-sitter Node to a ParsedNode
fn node_to_parsed_node(node: Node, kind: Kind) -> ParsedNode {
    ParsedNode {
        kind,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_row: node.start_position().row,
        start_column: node.start_position().column,
        end_row: node.end_position().row,
        end_column: node.end_position().column,
    }
}

/// Find all kUses nodes in the tree, similar to find_kuses_nodes in uses_section.rs
fn find_kuses_nodes(tree: &Tree) -> Vec<Node<'_>> {
    fn traverse<'a>(node: Node<'a>, nodes: &mut Vec<Node<'a>>) {
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

/// Transform a kUses node into a UsesSection, skipping if there are errors
fn transform_kuses_to_uses_section(kuses_node: Node) -> Option<UsesSection> {
    // Check if the starting node has an error
    if kuses_node.has_error() {
        return None;
    }

    // Get the parent node (should be declUses)
    let parent = kuses_node.parent()?;

    // Check parent for errors
    if parent.has_error() {
        return None;
    }

    let mut siblings = Vec::new();
    let mut semicolon_node = None;

    // Examine all children of the parent (siblings of kuses_node)
    for i in 0..parent.child_count() {
        if let Some(child) = parent.child(i) {
            // Check each sibling for errors
            if child.has_error() {
                return None;
            }

            // Skip the kUses node itself
            if child == kuses_node {
                continue;
            }

            // Look for the section terminator (could be semicolon or kEnd)
            if child.kind() == ";" {
                semicolon_node = Some(child);
            } else if child.kind() == "kEnd" {
                semicolon_node = Some(child);
            } else {
                // Skip comma separators between module names
                if child.kind() == "," {
                    continue;
                }

                // Classify other siblings
                let kind = match child.kind() {
                    "moduleName" | "identifier" => Kind::Module,
                    "comment" => Kind::Comment,
                    "pp" => Kind::Preprocessor,
                    _ => Kind::Module, // Default to module for other types
                };
                siblings.push(node_to_parsed_node(child, kind));
            }
        }
    }

    // Return parsed section if we found a terminator
    if let Some(semicolon) = semicolon_node {
        Some(UsesSection {
            uses: node_to_parsed_node(kuses_node, Kind::Uses),
            siblings,
            semicolon: node_to_parsed_node(semicolon, Kind::Semicolon),
        })
    } else {
        // No terminator found - treat as error
        None
    }
}

/// Parse source code string and return ParseResult
pub fn parse(source: &str) -> Result<ParseResult, DFixxerError> {
    let tree = parse_to_tree(source)?;
    let kuses_nodes = find_kuses_nodes(&tree);

    let mut uses_sections = Vec::new();

    for kuses_node in kuses_nodes {
        // Try to transform each kUses node, skip if there are errors
        if let Some(uses_section) = transform_kuses_to_uses_section(kuses_node) {
            uses_sections.push(uses_section);
        }
        // If transformation fails (returns None), we skip this uses section
    }

    Ok(ParseResult { uses_sections })
}
