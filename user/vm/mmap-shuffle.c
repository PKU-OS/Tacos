/* Creates a 128 kB file and repeatedly shuffles data in it
   through a memory mapping. */

#include "cksum.h"
#include "random.h"
#include "user.h"

#define SIZE (128 * 1024)

static char* buf = (char*)0x10000000;
static char zeros[SIZE];

void main() {
    size_t i;
    int fd;

    /* Create file, mmap. */
    assert((fd = open("buffer", O_CREATE | O_RDWR)) > 2);
    assert(write(fd, zeros, SIZE) == SIZE, "write zeros");
    assert(mmap(fd, buf) != MAP_FAILED, "mmap \"buffer\"");

    /* Initialize. */
    for (i = 0; i < SIZE; i++) buf[i] = i * 257;
    printf("init: cksum=%d", cksum(buf, SIZE));

    /* Shuffle repeatedly. */
    for (i = 0; i < 10; i++) {
        shuffle(buf, SIZE, 1);
        printf("shuffle %d: cksum=%d", i, cksum(buf, SIZE));
    }
}
