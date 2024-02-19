/** Tries to read from an invalid fd, which must fail with return value -1. */

#include <limits.h>

#include "user.h"

void main() {
    char buf = 123;
    assert(read(0xdeadbeaf, &buf, 1) == -1);
    assert(read(5, &buf, 1) == -1);
    assert(read(1234, &buf, 1) == -1);
    assert(read(-1, &buf, 1) == -1);
    assert(read(-1024, &buf, 1) == -1);
    assert(read(INT_MIN, &buf, 1) == -1);
    assert(read(INT_MAX, &buf, 1) == -1);
    assert(buf == 123);
}
