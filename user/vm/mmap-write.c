/* Writes to a file through a mapping, and unmaps the file,
   then reads the data in the file back using the read system
   call to verify. */

#include "sample.inc"
#include "user.h"

#define ACTUAL ((void*)0x10000000)

void main() {
    int fd, i;
    mapid_t map;
    char buf[1024];

    /* Write file via mmap. */
    assert((fd = open("sample.txt", O_RDWR)) > 2);
    assert((map = mmap(fd, ACTUAL)) != MAP_FAILED);

    memset(ACTUAL, 42, strlen(sample));
    munmap(map);

    /* Read back via read(). */
    read(fd, buf, strlen(sample));
    for (i = 0; i < strlen(sample); i++)
        assert(buf[i] == 42, "check that mmap write was successful");

    /* Write origin content back */
    write(fd, sample, strlen(sample));
    close(fd);
}
