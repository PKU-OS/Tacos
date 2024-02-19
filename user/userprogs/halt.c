/** Tests the halt system call. */

#include "user.h"

void main() {
    halt();
    panic("halt should not return");
}
