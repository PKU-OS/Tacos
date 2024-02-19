/** Closes a open file twice. The first close should be successful,
   whereas the second call should return -1 instead. */

#include "user.h"

void main() {
    int fd = open("sample.txt", 0);
    assert(close(fd) == 0, "%d is a valid open file descriptor", fd);
    assert(close(fd) == -1, "%d is closed and not valid", fd);
}
