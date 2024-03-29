/* Mmaps a 128 kB file "sorts" the bytes in it, using quick sort,
   a multi-pass divide and conquer algorithm.  */

#include "user.h"
#include "qsort.h"

void main(int argc, char* argv[]) {
    int fd;
    unsigned char* p = (unsigned char*)0x10000000;

    assert((fd = open(argv[1], O_RDWR)) > 2);
    assert(mmap(fd, p) != MAP_FAILED);
    qsort_bytes(p, 1024 * 128);

    exit(80);
}
