/* Try to write to the code segment using a system call.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    int fd;

    assert((fd = open("sample.txt", 0)) > 2);
    assert(read(fd, (void*)main, 1) == -1);
}
