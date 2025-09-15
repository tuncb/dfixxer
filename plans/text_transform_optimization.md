│ Performance Optimization Plan for transform_text.rs                                                                                                                         │ │
│ │                                                                                                                                                                             │ │
│ │ Performance Issues Found:                                                                                                                                                   │ │
│ │                                                                                                                                                                             │ │
│ │ 1. Redundant buffer selection logic                                                                                                                                         │ │
│ │   - active_buf() called 2-3 times per operator processing                                                                                                                   │ │
│ │   - Each call does a conditional check                                                                                                                                      │ │
│ │ 2. Excessive string introspection                                                                                                                                           │ │
│ │   - chars().last() called multiple times in helper functions                                                                                                                │ │
│ │   - Creates iterator each time just to get last char                                                                                                                        │ │
│ │ 3. Inefficient operator detection                                                                                                                                           │ │
│ │   - For '<' operator, handle_operator called twice (for <= and <>)                                                                                                          │ │
│ │   - Should detect all possibilities in one pass                                                                                                                             │ │
│ │ 4. Repeated peek() operations                                                                                                                                               │ │
│ │   - chars.peek().copied() called multiple times                                                                                                                             │ │
│ │   - Can be cached once per iteration                                                                                                                                        │ │
│ │ 5. Function call overhead                                                                                                                                                   │ │
│ │   - push_char closure called for every single character                                                                                                                     │ │
│ │   - Could batch character operations                                                                                                                                        │ │
│ │                                                                                                                                                                             │ │
│ │ Proposed Optimizations:                                                                                                                                                     │ │
│ │                                                                                                                                                                             │ │
│ │ 1. Cache buffer selection                                                                                                                                                   │ │
│ │   - Store buffer reference once per operator processing                                                                                                                     │ │
│ │   - Eliminate redundant if do_trim checks                                                                                                                                   │ │
│ │ 2. Track last character state                                                                                                                                               │ │
│ │   - Maintain a last_char variable instead of repeatedly calling chars().last()                                                                                              │ │
│ │   - Update it as we push characters                                                                                                                                         │ │
│ │ 3. Optimize operator detection                                                                                                                                              │ │
│ │   - Combine multi-character operator detection into single match                                                                                                            │ │
│ │   - Avoid multiple handle_operator calls                                                                                                                                    │ │
│ │ 4. Cache peek result                                                                                                                                                        │ │
│ │   - Store chars.peek().copied() once at start of processing                                                                                                                 │ │
│ │   - Reuse throughout operator handling                                                                                                                                      │ │
│ │ 5. Reduce function calls                                                                                                                                                    │ │
│ │   - Inline hot-path functions where appropriate                                                                                                                             │ │
│ │   - Consider macro for simple repeated patterns                                                                                                                             │ │
│ │ 6. Use more efficient string operations                                                                                                                                     │ │
│ │   - Use ends_with() for checking trailing whitespace                                                                                                                        │ │
│ │   - Batch string pushes where possible                                                                                                                                      │ │
│ │                                                                                                                                                                             │ │
│ │ Expected Performance Gains:                                                                                                                                                 │ │
│ │                                                                                                                                                                             │ │
│ │ - Reduced function call overhead (~10-15% improvement)                                                                                                                      │ │
│ │ - Fewer string allocations and introspections (~5-10% improvement)                                                                                                          │ │
│ │ - Better branch prediction from simplified logic (~5% improvement)                                                                                                          │ │
│ │ - Overall expected improvement: 20-30% for files with many operators