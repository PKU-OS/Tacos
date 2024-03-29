/* Uses a memory mapping to read a file. */

#include "sample.inc"
#include "user.h"

void main() {
    char* actual = (char*)0x10000000;
    int fd;
    mapid_t map;
    size_t i;

    assert((fd = open("sample.txt", 0)) > 2);
    assert((map = mmap(fd, actual)) != MAP_FAILED);

    /* Check that data is correct. */
    if (memcmp(actual, sample, strlen(sample))) panic("read of mmap'd file reported bad data");

    /* Verify that data is followed by zeros. */
    for (i = strlen(sample); i < 4096; i++)
        if (actual[i] != 0)
            panic("byte %d of mmap'd region has value %02hhx (should be 0)", i, actual[i]);

    munmap(map);
    close(fd);
}
