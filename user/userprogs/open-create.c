/** Creates a file by forwarding the O_CREATE flag. */

#include "user.h"

void main() {
    int fd;
    char* path = "non-exist.txt";
    assert(open(path, O_RDONLY) == -1, "file doesn't exist");
    assert((fd = open(path, O_CREATE)) > 2, "open should create a file with flag O_CREATE");
    close(fd);
}
