use crate::dfixxer_error::DFixxerError;
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;

/// Enum representing the kind of parsed node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Uses,
    Program,
    Unit,
    Semicolon,
    Module,
    Comment,
    Preprocessor,
}

/// Struct to store parsed text block information independent of tree-sitter types.
#[derive(Clone, PartialEq, Eq)]
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

impl std::fmt::Debug for ParsedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsedNode {{ kind: {:?}, start_byte: {}, end_byte: {}, start_row: {}, start_column: {}, end_row: {}, end_column: {} }}", 
               self.kind, self.start_byte, self.end_byte, self.start_row, self.start_column, self.end_row, self.end_column)
    }
}

/// Struct representing a code section (uses or program) in the parsed text.
/// The section type can be determined from the keyword's Kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSection {
    pub keyword: ParsedNode,
    pub siblings: Vec<ParsedNode>,
}

/// Struct representing the result of parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub code_sections: Vec<CodeSection>,
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

/// Traverse the AST and parse nodes of interest
fn traverse_and_parse<'a>(node: Node<'a>, code_sections: &mut Vec<CodeSection>) {
    match node.kind() {
        "kUses" => {
            // When we find a uses node, try to transform it into a CodeSection
            if let Some(code_section) = transform_keyword_to_code_section(node, Kind::Uses) {
                code_sections.push(code_section);
            }
            // Continue parsing after this uses section (no need to traverse children)
            return;
        }
        "kProgram" => {
            // When we find a program node, try to transform it into a CodeSection
            if let Some(code_section) = transform_keyword_to_code_section(node, Kind::Program) {
                code_sections.push(code_section);
            }
            // Continue parsing after this program statement (no need to traverse children)
            return;
        }
        "kUnit" => {
            // When we find a unit node, try to transform it into a CodeSection
            if let Some(code_section) = transform_keyword_to_code_section(node, Kind::Unit) {
                code_sections.push(code_section);
            }
            // Continue parsing after this unit statement (no need to traverse children)
            return;
        }
        _ => {
            // For other node types, continue traversing children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    traverse_and_parse(child, code_sections);
                }
            }
        }
    }
}


/// Generic transform function for kUses, kProgram, and kUnit nodes into a CodeSection
fn transform_keyword_to_code_section(keyword_node: Node, keyword_kind: Kind) -> Option<CodeSection> {
    // Check if the starting node has an error
    if keyword_node.has_error() {
        return None;
    }

    // Get the parent node (should be declUses or declProgram)
    let parent = keyword_node.parent()?;

    // Check parent for errors
    if parent.has_error() {
        return None;
    }

    let mut siblings = Vec::new();

    // Examine all children of the parent (siblings of keyword_node)
    for i in 0..parent.child_count() {
        if let Some(child) = parent.child(i) {
            // Check each sibling for errors
            if child.has_error() {
                return None;
            }

            // Skip the keyword node itself
            if child == keyword_node {
                continue;
            }

            // Handle semicolon and end markers
            if child.kind() == ";" {
                siblings.push(node_to_parsed_node(child, Kind::Semicolon));
            } else if child.kind() == "kEnd" {
                siblings.push(node_to_parsed_node(child, Kind::Semicolon));
            } else if child.kind() == "," {
                // Skip comma separators between module names
                continue;
            } else {
                // Classify other siblings
                let kind = match child.kind() {
                    "moduleName" | "identifier" => Kind::Module,
                    "comment" => Kind::Comment,
                    "pp" => Kind::Preprocessor,
                    _ => {
                        // For program and unit statements, skip other nodes like block, kEndDot, etc.
                        // For uses statements, default to module
                        match keyword_kind {
                            Kind::Program | Kind::Unit => continue, // Skip other nodes for program and unit statements
                            Kind::Uses => Kind::Module, // Default to module for uses statements
                            _ => continue,
                        }
                    }
                };
                siblings.push(node_to_parsed_node(child, kind));
            }
        }
    }

    Some(CodeSection {
        keyword: node_to_parsed_node(keyword_node, keyword_kind),
        siblings,
    })
}

/// Parse source code string and return ParseResult
pub fn parse(source: &str) -> Result<ParseResult, DFixxerError> {
    let tree = parse_to_tree(source)?;
    let mut code_sections = Vec::new();

    // Traverse the AST and collect all code sections
    traverse_and_parse(tree.root_node(), &mut code_sections);

    Ok(ParseResult { code_sections })
}

/// Parse the source, create the tree-sitter tree, and print each node's kind and text
pub fn parse_raw(source: &str) -> Result<(), DFixxerError> {
    let tree = parse_to_tree(source)?;
    let root = tree.root_node();
    fn print_node(node: tree_sitter::Node, depth: usize, source: &str) {
        let indent = "  ".repeat(depth);
        let kind = node.kind();
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        println!("{}Node kind: {} | Text: {}", indent, kind, text);
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                print_node(child, depth + 1, source);
            }
        }
    }
    // Skip printing the root node, print only its children
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            print_node(child, 0, source);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program_statement_legacy() {
        let source = r#"program myProgram;
begin
end."#;
        
        let result = parse(source).expect("Failed to parse");
        
        // Should have one code section
        assert_eq!(result.code_sections.len(), 1);
        
        let code_section = &result.code_sections[0];
        
        // Check keyword node
        assert_eq!(code_section.keyword.kind, Kind::Program);
        
        // Check siblings - should include module name and semicolon
        let has_module = code_section.siblings.iter().any(|s| s.kind == Kind::Module);
        let has_semicolon = code_section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
        
        assert!(has_module, "Should have module name in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");
        
        // Verify positions are reasonable
        assert_eq!(code_section.keyword.start_byte, 0);
    }

    #[test]
    fn test_parse_code_section_program() {
        let source = r#"program myProgram;
begin
end."#;
        
        let result = parse(source).expect("Failed to parse");
        
        // Should have one code section
        assert_eq!(result.code_sections.len(), 1);
        
        let code_section = &result.code_sections[0];
        
        // Check keyword node is program type
        assert_eq!(code_section.keyword.kind, Kind::Program);
        
        // Check siblings - should include module name and semicolon
        assert!(code_section.siblings.len() >= 1);
        
        // Find module and semicolon in siblings
        let has_module = code_section.siblings.iter().any(|s| s.kind == Kind::Module);
        let has_semicolon = code_section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
        
        assert!(has_module, "Should have module name in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");
    }

    #[test]
    fn test_parse_code_section_uses() {
        let source = r#"program myProgram;
uses
  UnitA,
  UnitB;
begin
end."#;
        
        let result = parse(source).expect("Failed to parse");
        
        // Should have two code sections (program and uses)
        assert_eq!(result.code_sections.len(), 2);
        
        // Find the uses section
        let uses_section = result.code_sections.iter()
            .find(|cs| cs.keyword.kind == Kind::Uses)
            .expect("Should have uses section");
        
        // Check siblings - should include modules and semicolon
        let module_count = uses_section.siblings.iter().filter(|s| s.kind == Kind::Module).count();
        let has_semicolon = uses_section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
        
        assert_eq!(module_count, 2, "Should have two modules in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");
    }

    #[test]
    fn test_parse_code_section_unit() {
        let source = r#"unit MyUnit;
interface
implementation
end."#;
        
        let result = parse(source).expect("Failed to parse");
        
        // Should have one code section (unit)
        assert_eq!(result.code_sections.len(), 1);
        
        let code_section = &result.code_sections[0];
        
        // Check keyword node is unit type
        assert_eq!(code_section.keyword.kind, Kind::Unit);
        
        // Check siblings - should include module name and semicolon
        let has_module = code_section.siblings.iter().any(|s| s.kind == Kind::Module);
        let has_semicolon = code_section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
        
        assert!(has_module, "Should have module name in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");
        
        // Verify positions are reasonable
        assert_eq!(code_section.keyword.start_byte, 0);
    }
}
