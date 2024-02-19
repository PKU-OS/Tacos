/* Verifies that overlapping memory mappings are disallowed. */

#include "sample.inc"
#include "user.h"

void main() {
    char* start = (char*)0x10000000;
    int fd[2];

    assert((fd[0] = open("zeros", 0)) > 2, "open \"zeros\" once");
    assert(mmap(fd[0], start) != MAP_FAILED, "mmap \"zeros\"");
    assert((fd[1] = open("zeros", 0)) > 2 && fd[0] != fd[1], "open \"zeros\" again");
    assert(mmap(fd[1], start + 4096) == MAP_FAILED, "try to mmap \"zeros\" again");
}
