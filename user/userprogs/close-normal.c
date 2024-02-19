/** Opens a file and then closes it. */

#include "user.h"

void main() {
    int fd;
    assert((fd = open("sample.txt", 0)) > 2);
    assert(close(fd) == 0);
}
