/** Opens a file and then runs a subprocess that tries to close
   the file.  (Pintos does not have inheritance of file handles,
   so this must fail.)  The parent process then attempts to use
   the file handle, which must succeed. */

#include "sample.inc"
#include "user.h"

void main() {
    int fd;

    assert((fd = open("sample.txt", O_RDWR)) > 1);

    char fd_str[5];
    itoa(fd_str, fd);
    const char* args[] = {"child-close", fd_str, NULL};

    assert(wait(exec("child-close", args)) == 64);

    check_file_handle(fd, "sample.txt", sample, sizeof sample - 1);
}
