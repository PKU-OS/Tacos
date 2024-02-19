/** Invokes a system call with `sp` set to a bad address.
   The process can still exit gracefully as long as it doesn't
   use sp. */

#include "user.h"

void main(void) {
    uint64 sp = r_sp();
    sp -= 16 * 4096;
    asm volatile("mv %0, sp; li a0, 0; j exit" ::"r"(sp));

    panic("unreachable");
}
