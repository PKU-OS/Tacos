/** Tries to write to an invalid fd, which must fail with return value -1. */

#include <limits.h>

#include "user.h"

void main() {
    char buf = 123;
    assert(write(0x20230823, &buf, 1) == -1);
    assert(write(7, &buf, 1) == -1);
    assert(write(2489, &buf, 1) == -1);
    assert(write(-11, &buf, 1) == -1);
    assert(write(-89, &buf, 1) == -1);
    assert(write(INT_MIN + 1, &buf, 1) == -1);
    assert(write(INT_MAX - 1, &buf, 1) == -1);
}
