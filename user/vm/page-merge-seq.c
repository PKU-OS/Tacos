/* Generates about 1 MB of random data that is then divided into
   16 chunks. A separate subprocess sorts each chunk in sequence.
   Then we merge the chunks and verify that the result is what it
   should be. */

#include "arc4.h"
#include "user.h"

/* This is the max file size for an older version of the Pintos
   file system that had 126 direct blocks each pointing to a
   single disk sector. We could raise it now. */
#define CHUNK_SIZE (126 * 512)
#define CHUNK_CNT 16                       /* Number of chunks. */
#define DATA_SIZE (CHUNK_CNT * CHUNK_SIZE) /* Buffer size. */

static uint8 buf1[DATA_SIZE], buf2[DATA_SIZE];
static size_t histogram[256];

/* Initialize buf1 with random data,
   then count the number of instances of each value within it. */
static void init(void) {
    struct arc4 arc4;
    size_t i;

    arc4_init(&arc4, "foobar", 6);
    arc4_crypt(&arc4, buf1, sizeof buf1);
    for (i = 0; i < sizeof buf1; i++) histogram[buf1[i]]++;
}

/* Sort each chunk of buf1 using a subprocess. */
static void sort_chunks(void) {
    size_t i;

    for (i = 0; i < CHUNK_CNT; i++) {
        pid_t child;
        int fd;

        /* Write this chunk to a file. */
        assert((fd = open("buffer", O_CREATE | O_WRONLY)) > 2);
        write(fd, buf1 + CHUNK_SIZE * i, CHUNK_SIZE);
        close(fd);

        /* Sort with subprocess. */
        const char* args[] = {"child-sort", "buffer", 0};
        assert((child = exec(args[0], args)) != -1);
        assert(wait(child) == 123, "wait for child-sort");

        /* Read chunk back from file. */
        assert((fd = open("buffer", 0)) > 2);
        read(fd, buf1 + CHUNK_SIZE * i, CHUNK_SIZE);
        close(fd);
    }
}

/* Merge the sorted chunks in buf1 into a fully sorted buf2. */
static void merge(void) {
    uint8* mp[CHUNK_CNT];
    size_t mp_left;
    uint8* op;
    size_t i;

    /* Initialize merge pointers. */
    mp_left = CHUNK_CNT;
    for (i = 0; i < CHUNK_CNT; i++) mp[i] = buf1 + CHUNK_SIZE * i;

    /* Merge. */
    op = buf2;
    while (mp_left > 0) {
        /* Find smallest value. */
        size_t min = 0;
        for (i = 1; i < mp_left; i++)
            if (*mp[i] < *mp[min]) min = i;

        /* Append value to buf2. */
        *op++ = *mp[min];

        /* Advance merge pointer.
           Delete this chunk from the set if it's emptied. */
        if ((++mp[min] - buf1) % CHUNK_SIZE == 0) mp[min] = mp[--mp_left];
    }
}

static void verify(void) {
    size_t buf_idx;
    size_t hist_idx;

    buf_idx = 0;
    for (hist_idx = 0; hist_idx < sizeof histogram / sizeof *histogram; hist_idx++) {
        while (histogram[hist_idx]-- > 0) {
            if (buf2[buf_idx] != hist_idx)
                panic("bad value %d in byte %d", buf2[buf_idx], buf_idx);
            buf_idx++;
        }
    }
}

void main() {
    init();
    sort_chunks();
    merge();
    verify();
}
