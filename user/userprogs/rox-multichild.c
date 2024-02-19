/** Ensure that the executable of a running process cannot be
   modified, even in the presence of multiple children. */

#include "user.h"

void main() {
    const char* args[] = {"child-rox", "5"};
    int fd, child, r;
    char buffer[16];

    /* Open child-rox, read from it, write back same data. */
    assert((fd = open("child-rox", O_RDWR)) > 2);

    r = read(fd, buffer, sizeof buffer);
    assert(r == (int)sizeof buffer);

    seek(fd, 0);

    r = write(fd, buffer, sizeof buffer);
    assert(r == (int)sizeof buffer);

    /* Execute child-rox and wait for it. */
    assert((child = exec(args[0], args)) >= 0);
    assert(wait(child) == 12);

    /* Write to child-rox again. */
    seek(fd, 0);
    r = write(fd, buffer, sizeof buffer);
    assert(r == (int)sizeof buffer);
    close(fd);
}
