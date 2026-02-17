use crate::parser::{InheritedExpansionCandidate, InheritedExpansionContext};
use crate::replacements::TextReplacement;

fn build_call_suffix(candidate: &InheritedExpansionCandidate) -> String {
    if candidate.arg_names.is_empty() {
        format!(" {}()", candidate.routine_name)
    } else {
        format!(
            " {}({})",
            candidate.routine_name,
            candidate.arg_names.join(", ")
        )
    }
}

/// Convert inherited expansion candidates into insertion replacements.
pub fn transform_inherited_calls(context: &InheritedExpansionContext) -> Vec<TextReplacement> {
    context
        .candidates
        .iter()
        .map(|candidate| TextReplacement {
            start: candidate.insert_at,
            end: candidate.insert_at,
            text: build_call_suffix(candidate),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_inherited_calls_with_args() {
        let context = InheritedExpansionContext {
            candidates: vec![InheritedExpansionCandidate {
                insert_at: 42,
                routine_name: "Create".to_string(),
                arg_names: vec!["AName".to_string(), "ACount".to_string()],
            }],
        };

        let replacements = transform_inherited_calls(&context);
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].start, 42);
        assert_eq!(replacements[0].end, 42);
        assert_eq!(replacements[0].text, " Create(AName, ACount)");
    }

    #[test]
    fn test_transform_inherited_calls_without_args() {
        let context = InheritedExpansionContext {
            candidates: vec![InheritedExpansionCandidate {
                insert_at: 11,
                routine_name: "Destroy".to_string(),
                arg_names: Vec::new(),
            }],
        };

        let replacements = transform_inherited_calls(&context);
        assert_eq!(replacements.len(), 1);
        assert_eq!(replacements[0].text, " Destroy()");
    }
}
