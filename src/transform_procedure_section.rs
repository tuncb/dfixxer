use crate::options::Options;
use crate::parser::{CodeSection, Kind};
use crate::replacements::TextReplacement;

/// Transform procedure/function declaration sections by adding parentheses after identifier
pub fn transform_procedure_section(
    code_section: &CodeSection,
    _options: &Options,
    _source: &str,
) -> Option<TextReplacement> {
    // Find the identifier in siblings
    let identifier_node = code_section.siblings.iter().find(|node| node.kind == Kind::Identifier)?;
    
    // Create the replacement: insert "()" after the identifier and before semicolon
    // We want to insert at the position right after the identifier ends
    Some(TextReplacement {
        start: identifier_node.end_byte,
        end: identifier_node.end_byte, // Insert, don't replace
        text: Some("()".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedNode, CodeSection, Kind};

    fn create_test_parsed_node(kind: Kind, start_byte: usize, end_byte: usize) -> ParsedNode {
        ParsedNode {
            kind,
            start_byte,
            end_byte,
            start_row: 0,
            start_column: start_byte,
            end_row: 0,
            end_column: end_byte,
        }
    }

    #[test]
    fn test_transform_procedure_section() {
        let source = "procedure Foo;";
        
        // Create test nodes
        let keyword_node = create_test_parsed_node(Kind::ProcedureDeclaration, 0, 9);
        let identifier_node = create_test_parsed_node(Kind::Identifier, 10, 13);
        let semicolon_node = create_test_parsed_node(Kind::Semicolon, 13, 14);
        
        let code_section = CodeSection {
            keyword: keyword_node,
            siblings: vec![identifier_node, semicolon_node],
        };
        
        let options = Options::default();
        let replacement = transform_procedure_section(&code_section, &options, source);
        
        assert!(replacement.is_some());
        let replacement = replacement.unwrap();
        assert_eq!(replacement.start, 13); // After "Foo"
        assert_eq!(replacement.end, 13);   // Insert, don't replace
        assert_eq!(replacement.text, Some("()".to_string()));
    }

    #[test]
    fn test_transform_function_section() {
        let source = "function Bar: Integer;";
        
        // Create test nodes - function should work the same as procedure
        let keyword_node = create_test_parsed_node(Kind::FunctionDeclaration, 0, 8);
        let identifier_node = create_test_parsed_node(Kind::Identifier, 9, 12);
        let semicolon_node = create_test_parsed_node(Kind::Semicolon, 21, 22);
        
        let code_section = CodeSection {
            keyword: keyword_node,
            siblings: vec![identifier_node, semicolon_node],
        };
        
        let options = Options::default();
        let replacement = transform_procedure_section(&code_section, &options, source);
        
        assert!(replacement.is_some());
        let replacement = replacement.unwrap();
        assert_eq!(replacement.start, 12); // After "Bar"
        assert_eq!(replacement.end, 12);   // Insert, don't replace
        assert_eq!(replacement.text, Some("()".to_string()));
    }
}