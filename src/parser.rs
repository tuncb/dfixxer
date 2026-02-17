use crate::dfixxer_error::DFixxerError;
use std::collections::HashSet;
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

/// Struct representing the result of parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub code_sections: Vec<CodeSection>,
}

/// Collected spacing context derived from the AST for operator-aware formatting.
#[derive(Debug, Clone, Default)]
pub struct SpacingContext {
    pub unary_minus_positions: HashSet<usize>,
    pub unary_plus_positions: HashSet<usize>,
    pub negative_literal_minus_positions: HashSet<usize>,
    pub positive_literal_plus_positions: HashSet<usize>,
    pub exponent_sign_positions: HashSet<usize>,
    pub generic_angle_positions: HashSet<usize>,
    pub expr_binary_lt_positions: HashSet<usize>,
    pub expr_binary_gt_positions: HashSet<usize>,
}

/// Candidate info for expanding a bare `inherited;` statement to an explicit call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InheritedExpansionCandidate {
    pub insert_at: usize,
    pub routine_name: String,
    pub arg_names: Vec<String>,
}

/// Collected context for inherited-call expansion transformations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InheritedExpansionContext {
    pub candidates: Vec<InheritedExpansionCandidate>,
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
        }
        "kProgram" => {
            // When we find a program node, try to transform it into a CodeSection
            if let Some(code_section) = transform_keyword_to_code_section(node, Kind::Program) {
                code_sections.push(code_section);
            }
            // Continue parsing after this program statement (no need to traverse children)
        }
        "kUnit" => {
            // When we find a unit node, try to transform it into a CodeSection
            if let Some(code_section) = transform_keyword_to_code_section(node, Kind::Unit) {
                code_sections.push(code_section);
            }
            // Continue parsing after this unit statement (no need to traverse children)
        }
        "kInterface" => {
            // When we find an interface node, transform it into a CodeSection (no siblings)
            if let Some(code_section) =
                transform_single_keyword_to_code_section(node, Kind::Interface)
            {
                code_sections.push(code_section);
            }
        }
        "kImplementation" => {
            // When we find an implementation node, transform it into a CodeSection (no siblings)
            if let Some(code_section) =
                transform_single_keyword_to_code_section(node, Kind::Implementation)
            {
                code_sections.push(code_section);
            }
        }
        "kInitialization" => {
            // When we find an initialization node, transform it into a CodeSection (no siblings)
            if let Some(code_section) =
                transform_single_keyword_to_code_section(node, Kind::Initialization)
            {
                code_sections.push(code_section);
            }
        }
        "kFinalization" => {
            // When we find a finalization node, transform it into a CodeSection (no siblings)
            if let Some(code_section) =
                transform_single_keyword_to_code_section(node, Kind::Finalization)
            {
                code_sections.push(code_section);
            }
        }
        "declProc" => {
            // Check if this is a procedure or function declaration without parentheses
            if let Some(code_section) = transform_procedure_declaration_to_code_section(node) {
                code_sections.push(code_section);
            }
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

fn collect_spacing_context(node: Node, source: &str, context: &mut SpacingContext) {
    match node.kind() {
        "genericTpl" | "typerefTpl" | "genericDot" | "exprTpl" => {
            let start = node.start_byte();
            let end = node.end_byte();
            if start < end && end <= source.len() {
                let text = &source[start..end];
                for (offset, ch) in text.char_indices() {
                    if ch == '<' || ch == '>' {
                        context.generic_angle_positions.insert(start + offset);
                    }
                }
            }
        }
        "exprUnary" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    match child.kind() {
                        "kSub" => {
                            context.unary_minus_positions.insert(child.start_byte());
                        }
                        "kAdd" => {
                            context.unary_plus_positions.insert(child.start_byte());
                        }
                        _ => {}
                    }
                }
            }
        }
        "exprBinary" => {
            if let Some(operator) = node.child_by_field_name("operator") {
                let start = operator.start_byte();
                let end = operator.end_byte();
                if start < end && end <= source.len() {
                    match &source[start..end] {
                        "<" | "<=" | "<>" => {
                            context.expr_binary_lt_positions.insert(start);
                        }
                        ">" | ">=" => {
                            context.expr_binary_gt_positions.insert(start);
                        }
                        _ => {}
                    }
                }
            }
        }
        "literalNumber" => {
            let start = node.start_byte();
            let end = node.end_byte();
            if start < end && end <= source.len() {
                let text = &source[start..end];
                if text.starts_with('-') {
                    context.negative_literal_minus_positions.insert(start);
                } else if text.starts_with('+') {
                    context.positive_literal_plus_positions.insert(start);
                }
                let mut chars = text.char_indices().peekable();
                while let Some((_, ch)) = chars.next() {
                    if (ch == 'e' || ch == 'E')
                        && let Some((sign_offset, sign_ch)) = chars.peek().copied()
                        && (sign_ch == '-' || sign_ch == '+')
                    {
                        context.exponent_sign_positions.insert(start + sign_offset);
                    }
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_spacing_context(child, source, context);
        }
    }
}

fn extract_routine_name_from_declproc(declproc_node: Node, source: &str) -> Option<String> {
    let mut fallback_identifier: Option<String> = None;

    for i in 0..declproc_node.child_count() {
        if let Some(child) = declproc_node.child(i) {
            match child.kind() {
                "identifier" => {
                    if fallback_identifier.is_none() {
                        fallback_identifier =
                            Some(source[child.start_byte()..child.end_byte()].to_string());
                    }
                }
                "genericDot" => {
                    let mut last_identifier: Option<String> = None;
                    for j in 0..child.child_count() {
                        if let Some(dot_child) = child.child(j)
                            && dot_child.kind() == "identifier"
                        {
                            last_identifier = Some(
                                source[dot_child.start_byte()..dot_child.end_byte()].to_string(),
                            );
                        }
                    }
                    if last_identifier.is_some() {
                        return last_identifier;
                    }
                }
                _ => {}
            }
        }
    }

    fallback_identifier
}

fn extract_arg_names_from_decl_args(declargs_node: Node, source: &str) -> Vec<String> {
    let mut arg_names = Vec::new();

    for i in 0..declargs_node.child_count() {
        if let Some(child) = declargs_node.child(i) {
            if child.kind() != "declArg" {
                continue;
            }

            for j in 0..child.child_count() {
                if let Some(arg_child) = child.child(j) {
                    if arg_child.kind() == ":" {
                        break;
                    }
                    if arg_child.kind() == "identifier" {
                        arg_names
                            .push(source[arg_child.start_byte()..arg_child.end_byte()].to_string());
                    }
                }
            }
        }
    }

    arg_names
}

fn bare_inherited_insert_at(statement: Node, source: &str) -> Option<usize> {
    if statement.kind() != "statement" || statement.has_error() {
        return None;
    }

    let mut inherited_node = None;
    let mut semicolon_node = None;

    for j in 0..statement.child_count() {
        if let Some(statement_child) = statement.child(j) {
            match statement_child.kind() {
                "inherited" => {
                    if inherited_node.is_some() {
                        return None;
                    }
                    inherited_node = Some(statement_child);
                }
                ";" => {
                    if semicolon_node.is_none() {
                        semicolon_node = Some(statement_child);
                    }
                }
                _ => {}
            }
        }
    }

    let (Some(inherited_node), Some(semicolon_node)) = (inherited_node, semicolon_node) else {
        return None;
    };

    // Bare inherited must only contain the keyword itself.
    if inherited_node.child_count() != 1 {
        return None;
    }
    let inherited_keyword = inherited_node.child(0)?;
    if inherited_keyword.kind() != "kInherited" {
        return None;
    }

    // Only expand when there is no inline content between `inherited` and `;`.
    let between = &source[inherited_node.end_byte()..semicolon_node.start_byte()];
    if !between.chars().all(char::is_whitespace) {
        return None;
    }

    Some(inherited_node.end_byte())
}

fn collect_bare_inherited_insert_points(node: Node, source: &str, insert_points: &mut Vec<usize>) {
    // Nested routine definitions should be handled by their own defProc traversal.
    if node.kind() == "defProc" {
        return;
    }

    if let Some(insert_at) = bare_inherited_insert_at(node, source) {
        insert_points.push(insert_at);
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_bare_inherited_insert_points(child, source, insert_points);
        }
    }
}

fn collect_inherited_candidates_from_defproc(
    defproc_node: Node,
    source: &str,
    context: &mut InheritedExpansionContext,
) {
    if defproc_node.has_error() {
        return;
    }

    let mut declproc_node = None;
    let mut block_node = None;

    for i in 0..defproc_node.child_count() {
        if let Some(child) = defproc_node.child(i) {
            match child.kind() {
                "declProc" => {
                    declproc_node = Some(child);
                }
                "block" => {
                    block_node = Some(child);
                }
                _ => {}
            }
        }
    }

    let (Some(declproc_node), Some(block_node)) = (declproc_node, block_node) else {
        return;
    };

    if declproc_node.has_error() || block_node.has_error() {
        return;
    }

    let Some(routine_name) = extract_routine_name_from_declproc(declproc_node, source) else {
        return;
    };

    let mut arg_names = Vec::new();
    for i in 0..declproc_node.child_count() {
        if let Some(child) = declproc_node.child(i)
            && child.kind() == "declArgs"
        {
            arg_names = extract_arg_names_from_decl_args(child, source);
            break;
        }
    }

    let mut insert_points = Vec::new();
    collect_bare_inherited_insert_points(block_node, source, &mut insert_points);
    for insert_at in insert_points {
        context.candidates.push(InheritedExpansionCandidate {
            insert_at,
            routine_name: routine_name.clone(),
            arg_names: arg_names.clone(),
        });
    }
}

fn collect_inherited_expansion_context(
    node: Node,
    source: &str,
    context: &mut InheritedExpansionContext,
) {
    if node.kind() == "defProc" {
        collect_inherited_candidates_from_defproc(node, source, context);
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_inherited_expansion_context(child, source, context);
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

    let mut routine_keyword_node = None;
    let mut routine_name_node = None;
    let mut has_decl_args = false;
    let mut semicolon_node = None;

    // Examine all children to find declaration heads without declArgs (no parentheses).
    // The routine name can be a plain identifier or a qualified genericDot.
    for i in 0..declproc_node.child_count() {
        if let Some(child) = declproc_node.child(i) {
            match child.kind() {
                "kProcedure" | "kFunction" | "kConstructor" | "kDestructor" | "kOperator" => {
                    routine_keyword_node = Some(child);
                }
                "identifier" => {
                    if routine_name_node.is_none() {
                        routine_name_node = Some(child);
                    }
                }
                "genericDot" => {
                    // Prefer qualified names (e.g. TMyClass.Create) when present.
                    routine_name_node = Some(child);
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
    if let (Some(routine_keyword), Some(routine_name), Some(semicolon)) =
        (routine_keyword_node, routine_name_node, semicolon_node)
        && !has_decl_args
    {
        // Constructors/destructors behave like procedures for this transform.
        // Operators behave like functions (may include a return type).
        let kind = match routine_keyword.kind() {
            "kProcedure" | "kConstructor" | "kDestructor" => Kind::ProcedureDeclaration,
            "kFunction" | "kOperator" => Kind::FunctionDeclaration,
            _ => return None,
        };

        let siblings = vec![
            node_to_parsed_node(routine_name, Kind::Identifier),
            node_to_parsed_node(semicolon, Kind::Semicolon),
        ];

        return Some(CodeSection {
            keyword: node_to_parsed_node(routine_keyword, kind),
            siblings,
        });
    }

    None
}

/// Parse source code string and return ParseResult
pub fn parse(source: &str) -> Result<ParseResult, DFixxerError> {
    let tree = parse_to_tree(source)?;
    let mut code_sections = Vec::new();

    // Traverse the AST and collect all code sections
    traverse_and_parse(tree.root_node(), &mut code_sections);

    Ok(ParseResult { code_sections })
}

/// Parse source code and collect parser contexts needed by transformations.
pub fn parse_with_contexts(
    source: &str,
) -> Result<(ParseResult, SpacingContext, InheritedExpansionContext), DFixxerError> {
    let tree = parse_to_tree(source)?;
    let mut code_sections = Vec::new();
    traverse_and_parse(tree.root_node(), &mut code_sections);

    let mut spacing_context = SpacingContext::default();
    collect_spacing_context(tree.root_node(), source, &mut spacing_context);

    let mut inherited_expansion_context = InheritedExpansionContext::default();
    collect_inherited_expansion_context(tree.root_node(), source, &mut inherited_expansion_context);

    Ok((
        ParseResult { code_sections },
        spacing_context,
        inherited_expansion_context,
    ))
}

/// Parse source code and also collect spacing context for AST-aware text transformations.
#[allow(dead_code)]
pub fn parse_with_spacing_context(
    source: &str,
) -> Result<(ParseResult, SpacingContext), DFixxerError> {
    let (parse_result, spacing_context, _) = parse_with_contexts(source)?;
    Ok((parse_result, spacing_context))
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
        println!(
            "{}Node kind: {} | Text: {}{}",
            indent, kind, text, error_info
        );
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
        assert!(!code_section.siblings.is_empty());

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
    fn test_parse_constructor_destructor_operator_without_parentheses() {
        let source = r#"unit TestCtorDtorOperator;
interface

type
  TMyRecord = record
    class operator Negative: TMyRecord;
  end;

  TMyClass = class
  public
    constructor Create;
    destructor Destroy;
  end;

implementation

constructor TMyClass.Create;
begin
end;

destructor TMyClass.Destroy;
begin
end;

class operator TMyRecord.Negative: TMyRecord;
begin
end;

end."#;

        let result = parse(source).expect("Failed to parse");

        let routine_sections: Vec<_> = result
            .code_sections
            .iter()
            .filter(|cs| {
                cs.keyword.kind == Kind::ProcedureDeclaration
                    || cs.keyword.kind == Kind::FunctionDeclaration
            })
            .collect();

        // Interface: Create, Destroy, Negative
        // Implementation: TMyClass.Create, TMyClass.Destroy, TMyRecord.Negative
        assert_eq!(routine_sections.len(), 6);

        let mut names = Vec::new();
        for section in &routine_sections {
            let identifier = section
                .siblings
                .iter()
                .find(|s| s.kind == Kind::Identifier)
                .expect("Should have identifier-like routine name in siblings");
            let has_semicolon = section.siblings.iter().any(|s| s.kind == Kind::Semicolon);
            assert!(has_semicolon, "Should have semicolon in siblings");
            names.push(source[identifier.start_byte..identifier.end_byte].to_string());
        }

        assert!(names.contains(&"Create".to_string()));
        assert!(names.contains(&"Destroy".to_string()));
        assert!(names.contains(&"Negative".to_string()));
        assert!(names.contains(&"TMyClass.Create".to_string()));
        assert!(names.contains(&"TMyClass.Destroy".to_string()));
        assert!(names.contains(&"TMyRecord.Negative".to_string()));
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
                cs.keyword.kind == Kind::ProcedureDeclaration
                    || cs.keyword.kind == Kind::FunctionDeclaration
            })
            .collect();

        assert_eq!(
            proc_func_sections.len(),
            0,
            "Should not detect procedures/functions that already have parentheses"
        );
    }

    #[test]
    fn test_parse_with_contexts_collects_inherited_expansion_with_args() {
        let source = r#"unit TestInherited;
interface

type
  TBase = class
  public
    constructor Create(const AName: string);
  end;

  TChild = class(TBase)
  public
    constructor Create(const AName: string);
  end;

implementation

constructor TChild.Create(const AName: string);
begin
  inherited;
end;

end."#;

        let (_, _, inherited_context) = parse_with_contexts(source).expect("Failed to parse");
        assert_eq!(inherited_context.candidates.len(), 1);

        let candidate = &inherited_context.candidates[0];
        let expected_insert = source
            .find("inherited")
            .expect("inherited should exist in test source")
            + "inherited".len();
        assert_eq!(candidate.insert_at, expected_insert);
        assert_eq!(candidate.routine_name, "Create");
        assert_eq!(candidate.arg_names, vec!["AName".to_string()]);
    }

    #[test]
    fn test_parse_with_contexts_collects_grouped_and_modifed_param_names() {
        let source = r#"unit TestInherited;
interface

type
  TBase = class
  public
    procedure Update(var AValue: Integer; out AErr: string; A, B: Integer);
  end;

  TChild = class(TBase)
  public
    procedure Update(var AValue: Integer; out AErr: string; A, B: Integer);
  end;

implementation

procedure TChild.Update(var AValue: Integer; out AErr: string; A, B: Integer);
begin
  inherited;
end;

end."#;

        let (_, _, inherited_context) = parse_with_contexts(source).expect("Failed to parse");
        assert_eq!(inherited_context.candidates.len(), 1);

        let candidate = &inherited_context.candidates[0];
        assert_eq!(candidate.routine_name, "Update");
        assert_eq!(
            candidate.arg_names,
            vec![
                "AValue".to_string(),
                "AErr".to_string(),
                "A".to_string(),
                "B".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_with_contexts_collects_inherited_expansion_without_args() {
        let source = r#"unit TestInherited;
interface

type
  TBase = class
  public
    destructor Destroy; virtual;
  end;

  TChild = class(TBase)
  public
    destructor Destroy; override;
  end;

implementation

destructor TChild.Destroy;
begin
  inherited;
end;

end."#;

        let (_, _, inherited_context) = parse_with_contexts(source).expect("Failed to parse");
        assert_eq!(inherited_context.candidates.len(), 1);

        let candidate = &inherited_context.candidates[0];
        assert_eq!(candidate.routine_name, "Destroy");
        assert!(candidate.arg_names.is_empty());
    }

    #[test]
    fn test_parse_with_contexts_skips_non_bare_inherited_forms() {
        let source = r#"unit TestInherited;
interface

type
  TBase = class
  public
    constructor Create(const AName: string);
  end;

  TChild = class(TBase)
  public
    constructor Create(const AName: string);
  end;

implementation

constructor TChild.Create(const AName: string);
begin
  inherited Create;
  inherited Create(AName);
end;

end."#;

        let (_, _, inherited_context) = parse_with_contexts(source).expect("Failed to parse");
        assert!(
            inherited_context.candidates.is_empty(),
            "Only bare inherited statements should produce expansion candidates"
        );
    }

    #[test]
    fn test_parse_with_contexts_collects_nested_bare_inherited() {
        let source = r#"unit TestInherited;
interface

type
  TBase = class
  public
    procedure DoWork(const AName: string);
  end;

  TChild = class(TBase)
  public
    procedure DoWork(const AName: string);
  end;

implementation

procedure TChild.DoWork(const AName: string);
begin
  if AName <> '' then
  begin
    inherited;
  end;
end;

end."#;

        let (_, _, inherited_context) = parse_with_contexts(source).expect("Failed to parse");
        assert_eq!(
            inherited_context.candidates.len(),
            1,
            "Nested bare inherited statements should be collected"
        );
        assert_eq!(inherited_context.candidates[0].routine_name, "DoWork");
        assert_eq!(
            inherited_context.candidates[0].arg_names,
            vec!["AName".to_string()]
        );
    }
}
