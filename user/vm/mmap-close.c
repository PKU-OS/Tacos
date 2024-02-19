/* Verifies that memory mappings persist after file close. */

#include "arc4.h"
#include "sample.inc"
#include "user.h"

#define ACTUAL ((void*)0x10000000)

void main() {
    int fd;
    mapid_t map;

    assert((fd = open("sample.txt", 0)) > 1, "open \"sample.txt\"");
    assert((map = mmap(fd, ACTUAL)) != MAP_FAILED, "mmap \"sample.txt\"");

    close(fd);

    if (memcmp(ACTUAL, sample, strlen(sample))) panic("read of mmap'd file reported bad data");

    munmap(map);
}
