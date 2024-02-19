/** Tries to close standard io streams. This is a dangerous action,
   but should not fail. */

#include "user.h"

void check_close(int fd) {
    if (close(fd) != 0) {
        exit(1);
    }
}

void main() {
    check_close(0);  // close stdin
    check_close(1);  // close stdout
    check_close(2);  // close stderr
}
