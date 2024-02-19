/** This program attempts to write to kernel memory. The kernel should
   kill this process after this malicious write operation. */

#include "user.h"

void main(void) {
    *(volatile int*)0xffffffc030000000 = 42;
    panic("should have exited");
}
