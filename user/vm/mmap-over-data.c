/* Verifies that mapping over the data segment is disallowed. */

#include "user.h"

static char x;

void main() {
    uint64 x_page = ROUND_DOWN(&x, 4096);
    int fd;

    assert((fd = open("sample.txt", 0)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, (void*)x_page) == MAP_FAILED, "try to mmap over data segment");
}
