/** Passes invalid arguments to system call `exec`. */

#include "user.h"

void main() {
    assert(exec("non-exist", NULL) == -1);

    const char* args[] = {"sample.txt", NULL};
    assert(exec(args[0], args) == -1, "\"sample.txt\" is not executable");
}
