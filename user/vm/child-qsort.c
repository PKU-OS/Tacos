/* Reads a 128 kB file onto the stack and "sorts" the bytes in
   it, using quick sort, a multi-pass divide and conquer
   algorithm.  The sorted data is written back to the same file
   in-place. */

#include "user.h"
#include "qsort.h"

void main(int argc, char* argv[]) {
    int fd;
    uint8 buf[128 * 1024];
    size_t size;

    assert((fd = open(argv[1], O_RDWR)) > 2);

    size = read(fd, buf, sizeof buf);
    qsort_bytes(buf, sizeof buf);
    seek(fd, 0);
    write(fd, buf, size);
    close(fd);

    exit(72);
}
