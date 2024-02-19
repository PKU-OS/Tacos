/* Verifies that memory mappings at address 0 are disallowed. */

#include "user.h"

void main() {
    int fd;

    assert((fd = open("sample.txt", 0)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, NULL) == MAP_FAILED, "try to mmap at address 0");
}
