/* Maps and unmaps a file and verifies that the mapped region is
   inaccessible afterward. */

#include "sample.inc"
#include "user.h"

#define ACTUAL ((void*)0x10000000)

void main() {
    int fd;
    mapid_t map;

    assert((fd = open("sample.txt", 0)) > 2);
    assert((map = mmap(fd, ACTUAL)) != MAP_FAILED);

    munmap(map);

    panic("unmapped memory is readable (%d)", *(int*)ACTUAL);
}
