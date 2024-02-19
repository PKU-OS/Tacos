/* Writes to a file through a mapping, and unmaps the file,
   then reads the data in the file back using the read system
   call to verify. */

#include "sample.inc"
#include "user.h"

#define ACTUAL ((void*)0x10000000)

void main() {
    int fd;
    mapid_t map;
    char buf[1024];

    /* Write file via mmap. */
    assert((fd = open("sample.txt", O_TRUNC | O_RDWR)) > 2);
    assert((map = mmap(fd, ACTUAL)) != MAP_FAILED);
    memcpy(ACTUAL, sample, strlen(sample));
    munmap(map);

    /* Read back via read(). */
    read(fd, buf, strlen(sample));
    assert(!memcmp(buf, sample, strlen(sample)), "compare read data against written data");
    close(fd);
}
