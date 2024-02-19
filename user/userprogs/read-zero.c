/** Try a 0-byte read, which should return 0 without reading
   anything. */

#include "user.h"

void main() {
    int fd, byte_cnt;
    char buf;

    assert((fd = open("sample.txt", O_RDONLY)));

    buf = 123;
    byte_cnt = read(fd, &buf, 0);
    assert(byte_cnt == 0, "read() returned %d instead of 0", byte_cnt);
    assert(buf == 123, "0-byte read() modified buffer");
    close(fd);
}
