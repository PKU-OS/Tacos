#include "sample.inc"
#include "user.h"

/* data will be at the end of .bss section */
static char data[4096 * 3];

void main() {
    int fd;

    /* open a path that's across the page boundary */
    char* path = "sample.txt";
    char* p = (char*)ROUND_UP(data, 4096);
    char* npath = p - strlen(path) / 2;
    strcpy(npath, path);
    assert((fd = open(npath, O_RDWR)) > 2);

    /* read across the page boundary */
    char* read_buf = (char*)(ROUND_UP(data + 4096, 4096) - sizeof sample / 2);
    assert(read(fd, read_buf, sizeof sample) == sizeof sample - 1);
    assert(strcmp(read_buf, sample) == 0);

    /* write across the page boundary */
    seek(fd, 0);
    assert(write(fd, read_buf, sizeof sample - 1) == sizeof sample - 1);
}
