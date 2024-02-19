/** This program attempts to read kernel memory, which makes the
   process killed by the kernel. */

#include "user.h"

void main() {
    printf("Congratulations - you have successfully read kernel memory: %d",
           *(int*)0xffffffc030000000);
    panic("should have exited");
}
