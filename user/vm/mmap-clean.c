/* Verifies that mmap'd regions are only written back on munmap
   if the data was actually modified in memory. */

#include "sample.inc"
#include "user.h"

void main() {
    static const char overwrite[] = "Now is the time for all good...";
    static char buffer[sizeof sample - 1];
    char* actual = (char*)0x54321000;
    int handle;
    mapid_t map;

    /* Open file, map, verify data. */
    assert((handle = open("sample.txt", O_RDWR)) > 2, "open \"sample.txt\"");
    assert((map = mmap(handle, actual)) != MAP_FAILED, "mmap \"sample.txt\"");
    if (memcmp(actual, sample, strlen(sample))) panic("read of mmap'd file reported bad data");

    /* Modify file. */
    assert(write(handle, overwrite, strlen(overwrite)) == (int)strlen(overwrite),
           "write \"sample.txt\"");

    /* Close mapping.  Data should not be written back, because we
       didn't modify it via the mapping. */
    printf("munmap \"sample.txt\"");
    munmap(map);

    /* Read file back. */
    printf("seek \"sample.txt\"");
    seek(handle, 0);
    assert(read(handle, buffer, sizeof buffer) == sizeof buffer, "read \"sample.txt\"");

    /* Verify that file overwrite worked. */
    if (memcmp(buffer, overwrite, strlen(overwrite)) ||
        memcmp(buffer + strlen(overwrite), sample + strlen(overwrite),
               strlen(sample) - strlen(overwrite))) {
        if (!memcmp(buffer, sample, strlen(sample))) {
            panic("munmap wrote back clean page");
        } else {
            panic("read surprising data from file");
        }
    } else
        printf("file change was retained after munmap");
}
