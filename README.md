# Tangle

> When your preprocessor and parser tangles together.

An experimental C parser that embeds preprocessor into parser.

This is meant to prototype "regional lexer" as a solution to seamlessly parse 
`#define` alised tokens and macro tokens without additional process stage.
The final shipment to production will be introduced in [sysprog21/shecc](https://github.com/sysprog21/shecc/)
once the prototype is finished and tested.
