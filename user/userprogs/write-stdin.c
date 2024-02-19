/** Try writing to fd 0 (stdin), which should just fail with return value -1. */

#include "user.h"

void main() {
    char buf = 123;
    assert(write(0, &buf, 1) == -1, "writing to stdin is an invalid action");
}
