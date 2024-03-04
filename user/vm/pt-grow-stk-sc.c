/* This test checks that the stack is properly extended even if
   the first access to a stack location occurs inside a system
   call. */

#include "sample.inc"
#include "user.h"

void main(void) {
    int fd;
    int slen = strlen(sample);
    char buf2[65536];

    /* Write file via write(). */
    assert(((fd = open("sample.txt", O_CREATE | O_WRONLY | O_TRUNC)) > 2));
    assert(write(fd, sample, slen) == slen);
    close(fd);

    /* Read back via read(). */
    assert((fd = open("sample.txt", 0)) > 2);
    assert(read(fd, buf2 + 32768, slen) == slen);

    assert(!memcmp(sample, buf2 + 32768, slen), "compare written data against read data");
    close(fd);
}
