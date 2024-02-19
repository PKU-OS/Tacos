/** Tests the exit system call. */

#include "user.h"

void main() {
    exit(0);
    panic("exit should not return");
}
