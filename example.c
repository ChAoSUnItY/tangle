#define CONSTANT_10 1 + 9
#define ADD(a, b) a + b
#define get(c) ADD(1, c + c)
#define MULTILINE(a, b) a + b \
    + 1

ADD(CONSTANT_10, 1);
ADD(100 + 2, get(1));
MULTILINE(1, 2);
/* 100 + 2 + 1 + 1 + 1; */