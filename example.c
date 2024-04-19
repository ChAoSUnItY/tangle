#define CONSTANT_10 1 + 9
#define ADD(a, b) a + b
#define get(c) ADD(1, c + c)

ADD(CONSTANT_10, 1);
ADD(100 + 2, get(1));
/* 100 + 2 + 1 + 1 + 1; */