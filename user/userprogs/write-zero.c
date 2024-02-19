/** Try a 0-byte write, which should return 0 without writing
   anything. */

#include "user.h"

void main() {
    int fd;
    char buf = 123;

    assert((fd = open("sample.txt", O_WRONLY)) > 2);
    assert(write(fd, &buf, 0) == 0);
    close(fd);
}
