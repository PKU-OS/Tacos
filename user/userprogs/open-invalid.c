/** Passes invalid arguments to system call `open`. */

#include "user.h"

void main() {
    assert(open(NULL, O_CREATE | O_TRUNC | O_WRONLY) == -1, "null pointer");
    assert(open("", O_CREATE | O_RDWR) == -1, "empty string");
    assert(open("non-exist.txt", 0) == -1, "file doesn't exist");
}
