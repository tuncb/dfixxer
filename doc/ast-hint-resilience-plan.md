# AST Hint Resilience Plan for Spacing Transformations

## Context
The current spacing logic relies on AST hints to avoid wrong formatting around:
- unary signs (`+`, `-`)
- exponent signs (`1E-12`, `1E+12`)
- generic angle brackets (`TArray<Integer>`)

On files where tree-sitter recovery drifts, some expected nodes are missing even when the file is otherwise valid Pascal. In those cases, formatter behavior can regress (for example: `TArray<Integer>` becoming `TArray < Integer >`).

## Why This Happens
- Hints are AST-only today (`src/parser.rs`, `SpacingContext`, `collect_spacing_context`).
- `transform_text` decisions for `<`, `>`, unary `+/-` depend on those hint sets.
- When parser recovery drops or misplaces nodes, those sets become incomplete.
- Current fallback tends to treat ambiguous tokens as regular operators.

## Recommended Structural Changes

### 1. Add Parse-Quality Metadata to `SpacingContext`
Extend context with parser reliability information:
- `error_ranges: Vec<(usize, usize)>`
- simple reliability flags or score (e.g. `has_errors`, `error_density_high`)

Purpose:
- let formatter know where AST hints are likely unreliable
- enable per-range fallback policy

### 2. Make Hinting Multi-Source (AST + Lexical)
Introduce a composable hint layer instead of direct AST sets only:
- `AstHintProvider` (existing behavior)
- `LexicalHintProvider` (new fallback)

Merge into one `HintMap` with confidence/source metadata per position.

Purpose:
- avoid single-point dependency on parser correctness
- support deterministic fallback in ambiguous regions

### 3. Add Lexical Classification for Ambiguous Tokens
Generalize the existing lexical exponent check approach to:
- generic angle detection (`<`, `>`)
- unary vs binary sign detection (`+`, `-`)

Purpose:
- keep behavior stable when AST misses specific nodes
- preserve good formatting outcomes on recovery-heavy files

### 4. Change Default Decision Policy in Low Confidence Areas
Current implicit policy behaves like: no hint -> operator formatting.

Recommended policy:
- no confident hint -> preserve original spacing (especially for `<` and `>`)
- only normalize when confident classification exists

Purpose:
- prevent destructive false positives
- prioritize safety under uncertainty

### 5. Gate AST Hints by Error Proximity
When token position is inside/near parser error ranges:
- degrade AST trust
- use lexical fallback or preserve-as-is

Purpose:
- limit drift propagation
- avoid misclassification after local parser recovery failures

### 6. Add Node-Type-Gated Spacing Rules
Apply different spacing policies based on AST node type, not only token kind.

Suggested rule matrix:
- `genericTpl`, `typerefTpl`, `exprTpl`:
  - tokens: `<`, `>`
  - rule: remove surrounding spaces (template/generic compact form)
- `exprBinary`:
  - tokens: comparison and arithmetic operators
  - rule: apply configured operator spacing (for example `lt`, `gt`, `lte`, `gte`, `add`, `sub`)
- `exprUnary`:
  - tokens: unary `+`, unary `-`
  - rule: compact unary form (for example `-1`, `+Foo`)
- `assignment`:
  - tokens: `:=`, `+=`, `-=`, `*=`, `/=`
  - rule: apply assignment spacing options
- declaration-only contexts (`defaultValue`, `declType`, etc.):
  - tokens: `=`
  - rule: use declaration-aware policy (or preserve unless explicitly configured)

Precedence order (important):
1. template/generic rule
2. assignment rule
3. unary rule
4. binary rule
5. preserve original spacing (fallback when context is missing or low confidence)

Purpose:
- support different spacing behavior per syntactic role
- reduce ambiguity where the same character appears in multiple contexts
- avoid applying expression rules inside template definitions

### 7. Refactor `transform_text` into a 3-Phase Pipeline
Current implementation is a large monolithic loop.

Target structure:
1. tokenize/state tracking
2. classify token roles (AST + lexical + confidence)
3. rewrite with spacing policy

Purpose:
- isolate classification from mutation
- make fallback logic explicit, testable, and maintainable

### 8. Add Regression Coverage for Recovery-Drift Inputs
Add end-to-end fixtures and focused unit tests for files where AST hints degrade.
Include the repro case:
- `debug-pascal-code/treesitter-error-repro_combined_recovery_drift_generics.pas`

Focus test assertions on:
- generic spacing preservation
- unary sign behavior
- exponent sign behavior
- boundary behavior after parser error sections

## Suggested Rollout Order
1. Parse-quality metadata + error ranges
2. Node-type-gated rules for high-risk ambiguous operators (`<`, `>`)
3. Conservative policy in low-confidence zones
4. Lexical fallback for generics (`<`, `>`)
5. Lexical fallback for unary `+/-`
6. Full pipeline refactor (tokenize/classify/rewrite)

## Expected Outcome
These changes reduce formatter dependence on perfect tree-sitter recovery and shift behavior toward safe, confidence-aware transformations. The result should be fewer spacing regressions on real-world Pascal files with parser drift.
