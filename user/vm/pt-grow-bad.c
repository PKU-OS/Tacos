/* Read from an address 4,096 bytes below the stack pointer.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    asm volatile(
        "li t0, -4096\n"
        "add t1, sp, t0\n"
        "ld t1, 0(t1)");
}
