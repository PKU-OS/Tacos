/** Passes invalid pointers to the exec, open, and write system call.
   These syscalls should return with value -1. */

#include "sample.inc"
#include "user.h"

void main() {
    int fd;
    char buf[256];

    assert((fd = open("sample.txt", O_RDWR)) > 2, "a normal system call");

    /* Passes NULL */
    assert(exec((void*)0xbad, (void*)0x478) == -1);
    assert(open(NULL, 0) == -1);
    assert(read(fd, NULL, sizeof buf));
    assert(write(1, NULL, 10) == -1);

    /* Passes arbitrary invalid pointers */
    assert(exec("child-simple", (void*)0xbad) == -1);
    assert(open((void*)0xdeadbeaf, O_CREATE) == -1);
    assert(read(fd, (void*)0xdeadbeaf, sizeof buf));
    assert(write(1, (void*)0xbad, 123) == -1);

    /* Passes pointers in the kernel space */
    assert(exec((void*)0xffffffc030000000, (void*)0xffffffc040000000) == -1);
    assert(open((void*)0xffffffc030000000, O_CREATE | O_TRUNC) == -1);
    assert(read(fd, (void*)0xffffffc000000000, sizeof buf));
    assert(write(1, (void*)0xffffffc030000000, 1234) == -1);

    /* After all the malicious system calls, process can still behave normally. */
    assert(read(fd, buf, sizeof buf) == sizeof sample - 1);
    assert(strcmp(buf, sample) == 0);
    assert(close(fd) == 0);
}
