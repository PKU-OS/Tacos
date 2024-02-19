/** Ensure that the executable of a running process cannot be
   modified. */

#include "user.h"

void main() {
    int fd, r;
    char buffer[16];

    fd = open("rox-simple", O_RDWR);
    if (fd <= 1) {
        panic("Failed to open \"rox-simple\"");
    }

    r = read(fd, buffer, sizeof buffer);
    assert(r == (int)sizeof buffer);
    assert(write(fd, buffer, sizeof buffer) == -1,
           "executable files should not be writable when running");

    close(fd);
}
