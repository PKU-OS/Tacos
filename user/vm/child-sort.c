/* Reads a 128 kB file into static data and "sorts" the bytes in
   it, using counting sort, a single-pass algorithm.  The sorted
   data is written back to the same file in-place. */

#include "user.h"

uint8 buf[128 * 1024];
size_t histogram[256];

void main(int argc, char* argv[]) {
    int fd;
    uint8* p;
    size_t size;
    size_t i;

    assert((fd = open(argv[1], O_RDWR)) > 2);

    size = read(fd, buf, sizeof buf);
    for (i = 0; i < size; i++) histogram[buf[i]]++;
    p = buf;
    for (i = 0; i < sizeof histogram / sizeof *histogram; i++) {
        size_t j = histogram[i];
        while (j-- > 0) *p++ = i;
    }
    seek(fd, 0);
    write(fd, buf, size);
    close(fd);
    exit(123);
}
