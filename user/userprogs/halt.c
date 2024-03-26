/** Tests the halt system call. */

#include "user.h"

void main() {
    printf("Goodbye, World!\n");
    halt();
    panic("halt should not return");
}
