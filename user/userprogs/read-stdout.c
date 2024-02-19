/** Try reading from fd 1 (stdout), which should fail with return value -1. */

#include "user.h"

void main() {
    char buf;
    assert(read(1, &buf, 1) == -1, "reading from the stdout");
}
