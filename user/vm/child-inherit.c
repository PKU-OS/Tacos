/* Child process for mmap-inherit test.
   Tries to write to a mapping present in the parent.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    memset((char*)0x54321000, 0, 4096);
    panic("child can modify parent's memory mappings");
}
