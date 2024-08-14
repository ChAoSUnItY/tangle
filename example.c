#define CONSTANT_10 1 + 9
#define ADD(a, b) a + b
#define get(c) ADD(1, c + c)
#define MULTILINE(a, b) a + b \
    + 1
#define VAR_FN(...) __VA_ARGS__
#define VAR_FN2(a, ...) a + __VA_ARGS__
#define STR(x) # x
#define IMM_STR(x) STR(x)

ADD(CONSTANT_10, 1);
ADD(100 + 2, get(1));
MULTILINE(1, 2);
VAR_FN(1, 2);
VAR_FN2(1, 2);
VAR_FN2(1);
STR(1);
STR(1 + 1);
STR(CONSTANT_10);
IMM_STR(CONSTANT_10);
