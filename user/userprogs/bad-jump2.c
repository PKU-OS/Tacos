/** This program attempts to execute code at a kernel virtual address. */

#include "user.h"

void main(void) {
    printf("Congratulations - you have successfully called kernel code: %d",
           ((int (*)(void))0xffffffc030000000)());
    panic("should have exited");
}
