# Plan: Add Space After Comma Transformation

## Overview
Add a new feature to automatically insert spaces after comma characters in Pascal/Delphi code.

## 1. Configuration Option
- Add `space_after_comma` boolean field to the `Options` struct in `src/options.rs`
- Default value: `false` (to maintain backward compatibility)
- This will allow users to enable/disable the transformation via config

## 2. Create New Transformation Module
- Create `src/transform_comma_spacing.rs` - dedicated module for comma spacing transformation
- Implement function that scans for commas in text and adds space if missing
- Handle edge cases like: already spaced commas, end of line commas, etc.

## 3. Track Unparsed Source Code
- Currently the system only processes parsed sections (uses, unit, program, etc.)
- Need to track and transform the remaining unparsed source code regions
- Modify `process_file()` in `main.rs` to collect unparsed regions between code sections

## 4. Apply to Two Contexts

### A. Unparsed Source Regions:
- Apply comma spacing to all code between the parsed sections
- This handles general Pascal/Delphi code outside of uses/unit/program sections

### B. Replacement Text (excluding uses sections):
- Apply comma spacing to generated replacement text from other transformations
- Skip uses sections since they have specific formatting rules that might conflict

## 5. Integration Points

### In `main.rs`:
- Collect unparsed regions between existing code sections
- Apply comma transformation to unparsed regions when enabled
- Apply comma transformation to replacement text from other transformations/

### In `replacements.rs`:
- Possibly extend `TextReplacement` to support chaining transformations
- Or create new composite replacements that merge comma spacing with existing replacements

## 6. Implementation Approach
1. After existing transformations create their replacements, identify unparsed regions
2. Apply comma spacing transformation to these regions if option enabled
3. For replacement text from other transformations (except uses sections), apply comma spacing
4. Merge all replacements while handling overlaps and conflicts

## 7. Testing
- Add unit tests for the comma spacing transformation
- Add integration tests with various Pascal code samples
- Test interaction with existing transformations

## Architecture Notes
This approach ensures comma spacing is applied comprehensively while maintaining the existing transformation architecture. The feature will work alongside existing transformations without interfering with their specific formatting rules.