# AST-Context-Gated Spacing Feasibility

## Summary
Applying spacing rules only when tokens are inside relevant AST nodes is feasible and is a strong direction for reducing false edits.

For the reported repro (`debug-pascal-code/treesitter-error-repro_combined_recovery_drift_generics.pas`), this approach would likely prevent the wrong generic rewrite (`TArray<Integer>` -> `TArray < Integer >`) because those tokens sit in parser recovery/error territory and should not be formatted unless context is trusted.

## Feasibility

1. `exprBinary` is explicitly defined in grammar (`misc/grammar.js:482`), so we can collect operator positions from AST and only apply spacing there.
2. Additional contexts are also explicit and can be mapped into position sets:
   - `exprUnary` (`misc/grammar.js:507`)
   - `assignment` (`misc/grammar.js:344`)
   - generic nodes: `exprTpl`, `typerefTpl`, `genericTpl` (`misc/grammar.js:473`, `misc/grammar.js:547`, `misc/grammar.js:564`)
3. This fits current architecture: `SpacingContext` is already built in `src/parser.rs` (`collect_spacing_context`).

## Critical Caveat

Using only `exprBinary` for all operators will regress behavior.

1. `=` is not only in `exprBinary`; it also appears in declarations:
   - `defaultValue` (`misc/grammar.js:640`)
   - `declType` (`misc/grammar.js:669`)
2. Generic `<` and `>` are often not `exprBinary`; they live in template/generic nodes.
3. Therefore, strict `exprBinary`-only gating would miss valid formatting cases and break current expectations.

## Will It Solve Wrong Changes in Poorly Parsed Regions?

Mostly yes, but not absolutely.

1. It will significantly reduce false positives by switching default behavior to "do not modify unless context is known and trusted."
2. It will likely solve the reported wrong generic spacing on the repro file.
3. It cannot guarantee perfect safety because tree-sitter may still produce incorrect but syntactically plausible AST nodes during recovery.

## Recommended Context-Gated Scope

Instead of `exprBinary`-only, use operator-specific context gates:

1. `<`, `>`, `<=`, `>=`, `<>`:
   - format only in `exprBinary` for comparisons
   - apply generic-angle handling only in generic/template nodes
2. `+`, `-`:
   - format in `exprBinary` and `exprUnary`
3. `*`, `/`:
   - format in `exprBinary`
4. `:=`, `+=`, `-=`, `*=`, `/=`:
   - format only in `assignment`
5. `,`, `;`, `:`:
   - keep mostly lexical/global handling
6. If no trusted context matches:
   - preserve original spacing

## Node-Type Rule Matrix (Concrete)

Use explicit per-node rules so the same token can behave differently by syntactic role.

1. Template/generic nodes: `genericTpl`, `typerefTpl`, `exprTpl`
   - Tokens: `<`, `>`
   - Rule: remove surrounding spaces (`TArray < Integer >` -> `TArray<Integer>`)
2. Binary-expression node: `exprBinary`
   - Tokens: `<`, `>`, `<=`, `>=`, `<>`, `+`, `-`, `*`, `/`, `=`
   - Rule: apply configured binary spacing options
3. Unary-expression node: `exprUnary`
   - Tokens: unary `+`, unary `-`
   - Rule: compact unary spacing (`- 1` -> `-1`, `+ Foo` -> `+Foo`)
4. Assignment node: `assignment`
   - Tokens: `:=`, `+=`, `-=`, `*=`, `/=`
   - Rule: apply assignment spacing options
5. Declaration contexts: `defaultValue`, `declType` (and similar declaration rules)
   - Token: `=`
   - Rule: declaration-aware handling (or preserve unless explicitly configured)
6. Non-expression punctuation
   - Tokens: `,`, `;`, `:`
   - Rule: keep lexical/global handling unless there is a strong reason to context-gate

## Precedence and Fallback

When multiple rules could apply, evaluate in this order:

1. template/generic rule
2. assignment rule
3. unary rule
4. binary rule
5. declaration-specific rule
6. preserve original spacing (no trusted context)

This precedence prevents expression-style spacing from leaking into template definitions and keeps ambiguous cases safe.

## Practical First Step

Implement context gating first for `<` and `>` only.

Why this first:
1. directly targets the reported corruption class
2. lower risk than full operator migration
3. easy to validate with focused regression fixtures

## Conclusion

Your proposed AST-context approach is feasible and valuable. It should reduce wrong changes substantially, especially after parser recovery drift. The safest version is context-gated-by-operator (not only `exprBinary`) with preserve-on-uncertain fallback.
