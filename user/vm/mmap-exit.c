/* Executes child-mm-wrt and verifies that the writes that should
   have occurred really did. */

#include "sample.inc"
#include "user.h"

void main() {
    pid_t child;

    /* Make child write file. */
    const char* args[] = {"child-mm-wrt", 0};
    assert((child = exec(args[0], args)) != -1, "exec \"child-mm-wrt\"");
    assert(wait(child) == 0, "wait for child (should return 0)");

    /* Check file contents. */
    check_file("sample.txt", sample, sizeof sample - 1);
}
