/** Opens 512 file handlers, which has to succeed. */

#include "user.h"

void main() {
    int fd1, fd2;

    fd1 = open("sample.txt", 0);
    for (int i = 0; i < 512 - 1; ++i) {
        assert(fd1 > 2);
        fd2 = open("sample.txt", 0);
        assert(fd1 != fd2);
        fd1 = fd2;
    }
}
