/* Maps a file into memory and runs child-inherit to verify that
   mappings are not inherited. */

#include "sample.inc"
#include "user.h"

void main() {
    char* actual = (char*)0x54321000;
    int fd;
    pid_t child;

    /* Open file, map, verify data. */
    assert((fd = open("sample.txt", O_RDONLY)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, actual) != MAP_FAILED, "mmap \"sample.txt\"");
    if (memcmp(actual, sample, strlen(sample))) panic("read of mmap'd file reported bad data");

    /* Spawn child and wait. */
    const char* args[] = {"child-inherit", 0};
    assert((child = exec(args[0], args)) != -1, "exec \"child-inherit\"");
    assert(wait(child) == -1, "wait for child (should return -1)");

    /* Verify data again. */
    assert(!memcmp(actual, sample, strlen(sample)),
           "checking that mmap'd file still has same data");
}
