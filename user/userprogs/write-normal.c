/** Try writing a file in the most normal way. */

#include "sample.inc"
#include "user.h"

void main() {
    int fd;

    assert((fd = open("test.txt", O_CREATE | O_WRONLY)) > 2);
    assert(write(fd, sample, sizeof sample - 1) == sizeof sample - 1);
    close(fd);
}
