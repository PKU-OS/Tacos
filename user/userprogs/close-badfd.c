/** Tries to close an invalid fd, which must either fail silently
   or terminate with exit code -1. */

#include "user.h"

void main() {
    int bad_fd = 0xdeadbeaf;
    assert(close(bad_fd) == -1, "%d isn't a valid open file descriptor.", bad_fd);
}
