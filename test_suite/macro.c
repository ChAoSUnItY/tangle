#define IDENTITY(a) a
#define IDENTITY_MULTILINE(a) a \

#define ADD(a, b) a + b
#define ADD_MULTILINE(a, b) a \
    + \
    b

IDENTITY(1);
IDENTITY(1 + 1);
IDENTITY_MULTILINE(1);
IDENTITY_MULTILINE(1 + 1);
ADD(1, 2);
ADD(1 + 1, 2);
ADD(1, 2 + 2);
ADD(1 + 1, 2 + 2);
ADD_MULTILINE(1, 2);
ADD_MULTILINE(1 + 1, 2);
ADD_MULTILINE(1, 2 + 2);
ADD_MULTILINE(1 + 1, 2 + 2);
