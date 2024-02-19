/** Opens a file to read and/or write. */

#include "sample.inc"
#include "user.h"

void main() {
    int fd;
    char buf[256];
    memset(buf, 0, sizeof buf);

    /* Read Only */
    assert((fd = open("sample.txt", O_RDONLY)) > 2);
    assert(read(fd, buf, sizeof buf) == (int)(sizeof sample - 1));
    assert(write(fd, buf, 1) == -1, "sample.txt was opened in read-only mode");
    assert(strcmp(buf, sample) == 0);
    close(fd);

    /* Write Only */
    assert((fd = open("tmp.txt", O_CREATE | O_WRONLY)) > 2);
    assert(write(fd, "abcdefg", 7) == 7);
    assert(read(fd, buf, sizeof buf) == -1);
    close(fd);

    /* Read and Write */
    assert((fd = open("sample.txt", O_RDWR)) > 2);
    assert(read(fd, buf, sizeof buf) == (int)(sizeof sample - 1));
    seek(fd, 0);
    assert(write(fd, buf, sizeof sample - 1) == (int)(sizeof sample - 1));
    seek(fd, 0);
    assert(read(fd, buf, sizeof buf) == (int)(sizeof sample - 1));
    assert(strcmp(buf, sample) == 0);
    close(fd);
}
