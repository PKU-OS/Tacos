/** Child process run by wait-killed test.
   Sets the stack pointer (%esp) to an invalid value and invokes
   a system call, which should then terminate the process with a
   -1 exit code. */

#include "user.h"

void main(void) {
    int* bad_ptr = (void*)0xdddaaabbb;
    *bad_ptr = 12345;

    panic("unreachable");
}
