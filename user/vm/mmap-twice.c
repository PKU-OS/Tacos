/* Maps the same file into memory twice and verifies that the
   same data is readable in both. */

#include "sample.inc"
#include "user.h"

void main() {
    char* actual[2] = {(char*)0x10000000, (char*)0x20000000};
    size_t i;
    int fd[2];

    for (i = 0; i < 2; i++) {
        assert((fd[i] = open("sample.txt", 0)) > 2, "open \"sample.txt\" #%d", i);
        assert(mmap(fd[i], actual[i]) != MAP_FAILED, "mmap \"sample.txt\" #%d at %p", i,
               (void*)actual[i]);
    }

    for (i = 0; i < 2; i++)
        assert(!memcmp(actual[i], sample, strlen(sample)), "compare mmap'd file %d against data",
               i);
}
