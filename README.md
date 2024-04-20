# Tangle

> When your preprocessor and parser tangles together.

An experimental C parser that embeds preprocessor into parser.

This is meant to prototype "regional lexer" as a solution to seamlessly parse 
`#define` alised tokens and macro tokens without additional process stage.
The final shipment to production will be introduced in [sysprog21/shecc](https://github.com/sysprog21/shecc/)
once the prototype is finished and tested.

## Todo List
- [x] 1-to-n `#define` alias
- [x] function-like macro
- [x] nested function-like macro
- [x] multiple line function-like macro (backed by `\` backslash character)
- [ ] `__VAR_ARGS__` parameter in function-like macro
- [ ] identifier concatentation (`##` concatentation operator)
- [ ] identifier stringfication (`#` stringification operator)
