/* Reads from a file into a bad address.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    int fd;

    assert((fd = open("sample.txt", 0)) > 2);
    read(fd, (char*)&fd - 4096, 1);
    panic("survived reading data into bad address");
}
