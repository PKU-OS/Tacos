/** Truncates a file, and then checks if it can be accessed properly. */

#include "user.h"

void main() {
    char buf[32];

    int fd1 = open("truncfile", O_CREATE | O_TRUNC | O_RDWR);
    assert(write(fd1, "abcd", 4) == 4);

    int fd2 = open("truncfile", O_TRUNC | O_RDWR);

    assert(write(fd2, "abcdef", 6) == 6);
    seek(fd2, 0);
    assert(read(fd2, buf, sizeof buf) == 6);  // fd2 was at position 0
    assert(read(fd1, buf, sizeof buf) == 2);  // fd1 was at position 4

    close(fd1);
    close(fd2);
}
