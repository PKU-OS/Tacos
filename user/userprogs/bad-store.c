/** This program attempts to write to memory at an address that is not mapped.
   The kernel should kill this process after this bad attempt. */

#include "user.h"

void main(void) {
    *(volatile int*)NULL = 42;
    panic("should have exited");
}
