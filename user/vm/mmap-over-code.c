/* Verifies that mapping over the code segment is disallowed. */

#include "user.h"

void main(void) {
    uint64 main_page = ROUND_DOWN(main, 4096);
    int fd;

    assert((fd = open("sample.txt", 0)) > 2, "open \"sample.txt\"");
    assert(mmap(fd, (void*)main_page) == MAP_FAILED, "try to mmap over code segment");
}
