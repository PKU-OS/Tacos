/* Verifies that mapping over the stack segment is disallowed. */

#include "user.h"

void main() {
    int fd;
    uint64 fd_page = ROUND_DOWN(&fd, 4096);

    assert((fd = open("sample.txt", 0)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, (void*)fd_page) == MAP_FAILED, "try to mmap over stack segment");
}
