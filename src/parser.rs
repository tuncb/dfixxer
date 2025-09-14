use crate::dfixxer_error::DFixxerError;
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_pascal::LANGUAGE;

/// Enum representing the kind of parsed node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Uses,
    Program,
    Unit,
    Interface,
    Implementation,
    Initialization,
    Finalization,
    Semicolon,
    Module,
    Comment,
    Preprocessor,
    ProcedureDeclaration,
    FunctionDeclaration,
    Identifier,
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
        write!(
            f,
            "ParsedNode {{ kind: {:?}, start_byte: {}, end_byte: {}, start_row: {}, start_column: {}, end_row: {}, end_column: {} }}",
            self.kind,
            self.start_byte,
            self.end_byte,
            self.start_row,
            self.start_column,
            self.end_row,
            self.end_column
        )
    }
}

/// Struct representing a code section (uses or program) in the parsed text.
/// The section type can be determined from the keyword's Kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSection {
    pub keyword: ParsedNode,
    pub siblings: Vec<ParsedNode>,
}

/// Struct representing an unparsed region in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsedRegion {
    /// Start byte position of the unparsed region
    pub start: usize,
    /// End byte position of the unparsed region
    pub end: usize,
}

/// Struct representing the result of parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub code_sections: Vec<CodeSection>,
    pub unparsed_regions: Vec<UnparsedRegion>,
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
        "kInterface" => {
            // When we find an interface node, transform it into a CodeSection (no siblings)
            if let Some(code_section) = transform_single_keyword_to_code_section(node, Kind::Interface) {
                code_sections.push(code_section);
            }
            return;
        }
        "kImplementation" => {
            // When we find an implementation node, transform it into a CodeSection (no siblings)
            if let Some(code_section) = transform_single_keyword_to_code_section(node, Kind::Implementation) {
                code_sections.push(code_section);
            }
            return;
        }
        "kInitialization" => {
            // When we find an initialization node, transform it into a CodeSection (no siblings)
            if let Some(code_section) = transform_single_keyword_to_code_section(node, Kind::Initialization) {
                code_sections.push(code_section);
            }
            return;
        }
        "kFinalization" => {
            // When we find a finalization node, transform it into a CodeSection (no siblings)
            if let Some(code_section) = transform_single_keyword_to_code_section(node, Kind::Finalization) {
                code_sections.push(code_section);
            }
            return;
        }
        "declProc" => {
            // Check if this is a procedure or function declaration without parentheses
            if let Some(code_section) = transform_procedure_declaration_to_code_section(node) {
                code_sections.push(code_section);
            }
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
fn transform_keyword_to_code_section(
    keyword_node: Node,
    keyword_kind: Kind,
) -> Option<CodeSection> {
    // Check if the starting node has an error
    if keyword_node.has_error() {
        return None;
    }

    // Get the parent node (should be declUses or declProgram)
    let parent = keyword_node.parent()?;

    // Check parent for errors, but skip for unit and program as they may cover the whole file
    if parent.has_error() && keyword_kind == Kind::Uses {
        return None;
    }

    let mut siblings = Vec::new();
    let mut found_module = false;

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
                // For unit and program sections, stop after the first semicolon that follows a module
                if (keyword_kind == Kind::Unit || keyword_kind == Kind::Program) && found_module {
                    break;
                }
            } else if child.kind() == "kEnd" {
                siblings.push(node_to_parsed_node(child, Kind::Semicolon));
                // For unit and program sections, stop after the first end marker that follows a module
                if (keyword_kind == Kind::Unit || keyword_kind == Kind::Program) && found_module {
                    break;
                }
            } else if child.kind() == "," {
                // Skip comma separators between module names
                continue;
            } else {
                // Classify other siblings
                let kind = match child.kind() {
                    "moduleName" | "identifier" => {
                        found_module = true;
                        Kind::Module
                    }
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

/// Transform function for single keyword nodes (interface, implementation, initialization, finalization)
/// These nodes don't have meaningful siblings, so they create empty CodeSections with just the keyword
fn transform_single_keyword_to_code_section(
    keyword_node: Node,
    keyword_kind: Kind,
) -> Option<CodeSection> {
    // Check if the node has an error
    if keyword_node.has_error() {
        return None;
    }

    Some(CodeSection {
        keyword: node_to_parsed_node(keyword_node, keyword_kind),
        siblings: Vec::new(), // No siblings for these single-word sections
    })
}

/// Transform function for procedure/function declarations without parentheses
/// These are `declProc` nodes that contain kProcedure/kFunction -> identifier -> ; (no declArgs)
fn transform_procedure_declaration_to_code_section(declproc_node: Node) -> Option<CodeSection> {
    // Check if the node has an error
    if declproc_node.has_error() {
        return None;
    }

    let mut proc_or_func_node = None;
    let mut identifier_node = None;
    let mut has_decl_args = false;
    let mut semicolon_node = None;

    // Examine all children to find the pattern: kProcedure/kFunction -> identifier -> ; (no declArgs)
    for i in 0..declproc_node.child_count() {
        if let Some(child) = declproc_node.child(i) {
            match child.kind() {
                "kProcedure" | "kFunction" => {
                    proc_or_func_node = Some(child);
                }
                "identifier" => {
                    identifier_node = Some(child);
                }
                "declArgs" => {
                    has_decl_args = true; // This procedure/function already has parentheses
                }
                ";" => {
                    semicolon_node = Some(child);
                }
                _ => {} // Skip other nodes like return types
            }
        }
    }

    // Only process if we have the pattern without declArgs
    if let (Some(proc_func), Some(identifier), Some(semicolon)) = 
        (proc_or_func_node, identifier_node, semicolon_node) {
        if !has_decl_args {
            // Determine if it's a procedure or function
            let kind = if proc_func.kind() == "kProcedure" {
                Kind::ProcedureDeclaration
            } else {
                Kind::FunctionDeclaration
            };

            let mut siblings = Vec::new();
            siblings.push(node_to_parsed_node(identifier, Kind::Identifier));
            siblings.push(node_to_parsed_node(semicolon, Kind::Semicolon));

            return Some(CodeSection {
                keyword: node_to_parsed_node(proc_func, kind),
                siblings,
            });
        }
    }

    None
}

/// Calculate unparsed regions based on CodeSections
/// An unparsed region is any part of the source that is not covered by a CodeSection
fn calculate_unparsed_regions(code_sections: &[CodeSection], source_len: usize) -> Vec<UnparsedRegion> {
    if code_sections.is_empty() {
        // If no code sections, entire source is unparsed
        if source_len > 0 {
            return vec![UnparsedRegion { start: 0, end: source_len }];
        } else {
            return vec![];
        }
    }

    let mut unparsed_regions = Vec::new();

    // Collect all parsed regions (start, end) from CodeSections
    let mut parsed_regions: Vec<(usize, usize)> = Vec::new();

    for section in code_sections {
        // Get the extent of the entire code section
        let mut min_start = section.keyword.start_byte;
        let mut max_end = section.keyword.end_byte;

        // Include all siblings in the parsed region
        for sibling in &section.siblings {
            min_start = min_start.min(sibling.start_byte);
            max_end = max_end.max(sibling.end_byte);
        }

        parsed_regions.push((min_start, max_end));
    }

    // Sort parsed regions by start position
    parsed_regions.sort_by_key(|&(start, _)| start);

    // Merge overlapping parsed regions
    let mut merged_parsed: Vec<(usize, usize)> = Vec::new();
    for (start, end) in parsed_regions {
        if let Some((_, last_end)) = merged_parsed.last_mut() {
            if *last_end >= start {
                // Regions overlap or are adjacent, merge them
                *last_end = (*last_end).max(end);
            } else {
                merged_parsed.push((start, end));
            }
        } else {
            merged_parsed.push((start, end));
        }
    }

    // Now find gaps between parsed regions - these are unparsed regions
    let mut current_pos = 0;

    for (parsed_start, parsed_end) in merged_parsed {
        // If there's a gap before this parsed region, it's unparsed
        if current_pos < parsed_start {
            unparsed_regions.push(UnparsedRegion {
                start: current_pos,
                end: parsed_start,
            });
        }
        current_pos = parsed_end;
    }

    // If there's remaining content after the last parsed region
    if current_pos < source_len {
        unparsed_regions.push(UnparsedRegion {
            start: current_pos,
            end: source_len,
        });
    }

    unparsed_regions
}

/// Parse source code string and return ParseResult
pub fn parse(source: &str) -> Result<ParseResult, DFixxerError> {
    let tree = parse_to_tree(source)?;
    let mut code_sections = Vec::new();

    // Traverse the AST and collect all code sections
    traverse_and_parse(tree.root_node(), &mut code_sections);

    // Calculate unparsed regions based on the code sections
    let unparsed_regions = calculate_unparsed_regions(&code_sections, source.len());

    Ok(ParseResult {
        code_sections,
        unparsed_regions,
    })
}

/// Parse the source, create the tree-sitter tree, and print each node's kind and text
pub fn parse_raw(source: &str) -> Result<(), DFixxerError> {
    let tree = parse_to_tree(source)?;
    let root = tree.root_node();
    fn print_node(node: tree_sitter::Node, depth: usize, source: &str) {
        let indent = "  ".repeat(depth);
        let kind = node.kind();
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        let error_info = if node.has_error() { " | ERROR" } else { "" };
        println!("{}Node kind: {} | Text: {}{}", indent, kind, text, error_info);
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
        let has_semicolon = code_section
            .siblings
            .iter()
            .any(|s| s.kind == Kind::Semicolon);

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
        let has_semicolon = code_section
            .siblings
            .iter()
            .any(|s| s.kind == Kind::Semicolon);

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
        let uses_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Uses)
            .expect("Should have uses section");

        // Check siblings - should include modules and semicolon
        let module_count = uses_section
            .siblings
            .iter()
            .filter(|s| s.kind == Kind::Module)
            .count();
        let has_semicolon = uses_section
            .siblings
            .iter()
            .any(|s| s.kind == Kind::Semicolon);

        assert_eq!(module_count, 2, "Should have two modules in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");
    }

    #[test]
    fn test_parse_code_section_unit() {
        let source = r#"UnIT   ex2 ;
interface
implementation
end."#;

        let result = parse(source).expect("Failed to parse");

        // Should have three code sections (unit, interface, implementation)
        assert_eq!(result.code_sections.len(), 3);

        // Find the unit section
        let unit_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Unit)
            .expect("Should have unit section");

        // Check keyword node is unit type
        assert_eq!(unit_section.keyword.kind, Kind::Unit);

        // Check siblings - should include module name and semicolon
        let has_module = unit_section.siblings.iter().any(|s| s.kind == Kind::Module);
        let has_semicolon = unit_section
            .siblings
            .iter()
            .any(|s| s.kind == Kind::Semicolon);

        assert!(has_module, "Should have module name in siblings");
        assert!(has_semicolon, "Should have semicolon in siblings");

        // Verify positions are reasonable
        assert_eq!(unit_section.keyword.start_byte, 0);
    }

    #[test]
    fn test_parse_interface_section() {
        let source = r#"unit MyUnit;
interface
implementation
end."#;

        let result = parse(source).expect("Failed to parse");

        // Should have three code sections (unit, interface, implementation)
        assert_eq!(result.code_sections.len(), 3);

        // Find the interface section
        let interface_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Interface)
            .expect("Should have interface section");

        // Interface sections should have no siblings
        assert_eq!(interface_section.siblings.len(), 0);
    }

    #[test]
    fn test_parse_implementation_section() {
        let source = r#"unit MyUnit;
interface
implementation
end."#;

        let result = parse(source).expect("Failed to parse");

        // Find the implementation section
        let impl_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Implementation)
            .expect("Should have implementation section");

        // Implementation sections should have no siblings
        assert_eq!(impl_section.siblings.len(), 0);
    }

    #[test]
    fn test_parse_initialization_finalization_sections() {
        let source = r#"unit MyUnit;
interface
implementation
initialization
finalization
end."#;

        let result = parse(source).expect("Failed to parse");

        // Find the initialization section
        let init_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Initialization);

        // Find the finalization section
        let final_section = result
            .code_sections
            .iter()
            .find(|cs| cs.keyword.kind == Kind::Finalization);

        if let Some(init) = init_section {
            assert_eq!(init.siblings.len(), 0);
        }

        if let Some(final_sec) = final_section {
            assert_eq!(final_sec.siblings.len(), 0);
        }
    }

    #[test]
    fn test_parse_procedure_without_parentheses() {
        let source = r#"unit TestProcedures;
interface
procedure Foo;
implementation
procedure Foo;
begin
end;
end."#;

        let result = parse(source).expect("Failed to parse");

        // Find procedure declaration sections
        let procedure_sections: Vec<_> = result
            .code_sections
            .iter()
            .filter(|cs| cs.keyword.kind == Kind::ProcedureDeclaration)
            .collect();

        // Should have two procedure declarations (interface and implementation)
        assert_eq!(procedure_sections.len(), 2);

        for section in &procedure_sections {
            // Each should have identifier and semicolon in siblings
            let has_identifier = section.siblings.iter().any(|s| s.kind == Kind::Identifier);
            let has_semicolon = section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
            
            assert!(has_identifier, "Should have identifier in siblings");
            assert!(has_semicolon, "Should have semicolon in siblings");
        }
    }

    #[test]
    fn test_parse_function_without_parentheses() {
        let source = r#"unit TestFunctions;
interface
function Bar: Integer;
implementation
function Bar: Integer;
begin
  Result := 42;
end;
end."#;

        let result = parse(source).expect("Failed to parse");

        // Find function declaration sections
        let function_sections: Vec<_> = result
            .code_sections
            .iter()
            .filter(|cs| cs.keyword.kind == Kind::FunctionDeclaration)
            .collect();

        // Should have two function declarations (interface and implementation)
        assert_eq!(function_sections.len(), 2);

        for section in &function_sections {
            // Each should have identifier and semicolon in siblings
            let has_identifier = section.siblings.iter().any(|s| s.kind == Kind::Identifier);
            let has_semicolon = section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
            
            assert!(has_identifier, "Should have identifier in siblings");
            assert!(has_semicolon, "Should have semicolon in siblings");
        }
    }

    #[test]
    fn test_parse_procedures_with_parentheses_not_detected() {
        let source = r#"unit TestProcedures;
interface
procedure WithParams(x: Integer);
function WithParamsAndReturn(x: Integer): String;
implementation
end."#;

        let result = parse(source).expect("Failed to parse");

        // Should not detect any procedure/function declaration sections
        // since they already have parentheses
        let proc_func_sections: Vec<_> = result
            .code_sections
            .iter()
            .filter(|cs| {
                cs.keyword.kind == Kind::ProcedureDeclaration || cs.keyword.kind == Kind::FunctionDeclaration
            })
            .collect();

        assert_eq!(proc_func_sections.len(), 0, "Should not detect procedures/functions that already have parentheses");
    }

    #[test]
    fn test_unparsed_regions_simple() {
        let source = r#"program MyProgram;
uses
  UnitA,
  UnitB;
begin
  WriteLn('Hello');
end."#;

        let result = parse(source).expect("Failed to parse");

        // The unparsed regions should include the begin..end block
        assert!(!result.unparsed_regions.is_empty(), "Should have unparsed regions");

        // The program declaration and uses section should be parsed
        assert_eq!(result.code_sections.len(), 2);

        // Find where the uses section ends
        let uses_section = result.code_sections.iter()
            .find(|cs| cs.keyword.kind == Kind::Uses)
            .expect("Should have uses section");

        let mut uses_end = uses_section.keyword.end_byte;
        for sibling in &uses_section.siblings {
            uses_end = uses_end.max(sibling.end_byte);
        }

        // There should be an unparsed region after the uses section
        let has_unparsed_after_uses = result.unparsed_regions.iter()
            .any(|r| r.start >= uses_end);

        assert!(has_unparsed_after_uses, "Should have unparsed region after uses section");
    }

    #[test]
    fn test_unparsed_regions_empty_file() {
        let source = "";

        let result = parse(source).expect("Failed to parse");

        // Empty file should have no unparsed regions
        assert_eq!(result.unparsed_regions.len(), 0);
        assert_eq!(result.code_sections.len(), 0);
    }

    #[test]
    fn test_unparsed_regions_only_comments() {
        let source = r#"// This is just a comment
{ Another comment }"#;

        let result = parse(source).expect("Failed to parse");

        // File with only comments should be entirely unparsed
        assert_eq!(result.code_sections.len(), 0);
        assert_eq!(result.unparsed_regions.len(), 1);
        assert_eq!(result.unparsed_regions[0].start, 0);
        assert_eq!(result.unparsed_regions[0].end, source.len());
    }

    #[test]
    fn test_unparsed_regions_between_sections() {
        let source = r#"unit MyUnit;
interface
uses
  System;
const
  MyConst = 42;
implementation
uses
  SysUtils;
end."#;

        let result = parse(source).expect("Failed to parse");

        // Should have unit, interface, uses (interface), implementation, uses (implementation) sections
        assert!(result.code_sections.len() >= 4);

        // Should have unparsed regions for const section and end.
        assert!(!result.unparsed_regions.is_empty(), "Should have unparsed regions");
    }
}
