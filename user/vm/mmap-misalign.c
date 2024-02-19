/* Verifies that misaligned memory mappings are disallowed. */

#include "user.h"

void main() {
    int fd;

    assert((fd = open("sample.txt", 0)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, (void*)0x10001234) == MAP_FAILED, "try to mmap at misaligned address");
}
