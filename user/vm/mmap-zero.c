/* Tries to map a zero-length file, which may or may not work but
   should not terminate the process or crash.
   Then dereferences the address that we tried to map,
   and the process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    char* data = (char*)0x7f000000;
    int fd;

    assert((fd = open("empty", O_CREATE | O_TRUNC | O_RDWR)) > 2, "open \"empty\"");

    /* Calling mmap() might succeed or fail.  We don't care. */
    printf("mmap \"empty\"");
    mmap(fd, data);

    /* Regardless of whether the call worked, *data should cause
       the process to be terminated. */
    panic("unmapped memory is readable (%d)", *data);
}
