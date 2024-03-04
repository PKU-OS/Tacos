/* Child process of mmap-exit.
   Mmaps a file and writes to it via the mmap'ing, then exits
   without calling munmap.  The data in the mapped region must be
   written out at program termination. */

#include "sample.inc"
#include "user.h"

#define ACTUAL ((void*)0x10000000)

void main() {
    int fd;

    assert((fd = open("sample.txt", O_WRONLY)) > 1);
    assert(mmap(fd, ACTUAL) != MAP_FAILED, "mmap \"sample.txt\"");
    memcpy(ACTUAL, sample, sizeof sample);
}
