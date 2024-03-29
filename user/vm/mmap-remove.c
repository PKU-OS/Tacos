/* Deletes and closes file that is mapped into memory
   and verifies that it can still be read through the mapping. */

#include "sample.inc"
#include "user.h"

void main() {
    char* actual = (char*)0x10000000;
    int fd;
    mapid_t map;
    size_t i;

    /* Map file. */
    assert((fd = open("sample.txt", O_RDWR)) > 2);
    assert((map = mmap(fd, actual)) != MAP_FAILED, "mmap \"sample.txt\"");

    /* Close file and delete it. */
    close(fd);
    assert(remove("sample.txt") == 0, "remove \"sample.txt\"");
    assert(open("sample.txt", 0) == -1, "try to open \"sample.txt\"");

    /* Create a new file in hopes of overwriting data from the old
       one, in case the file system has incorrectly freed the
       file's data. */
    int fd2;
    char buf[4096 * 8];
    assert((fd2 = open("another", O_CREATE | O_TRUNC | O_WRONLY)));
    assert(write(fd2, buf, 4096 * 8) == 4096 * 8);

    /* Check that mapped data is correct. */
    if (memcmp(actual, sample, strlen(sample))) panic("read of mmap'd file reported bad data");

    /* Verify that data is followed by zeros. */
    for (i = strlen(sample); i < 4096; i++)
        if (actual[i] != 0)
            panic("byte %d of mmap'd region has value %02hhx (should be 0)", i, actual[i]);

    munmap(map);
}
