/** Invokes an open system call with the path string straddling a
   page boundary such that the first byte of the string is valid
   but the remainder of the string is in invalid memory. Similar
   to passing an invalid pointer to `open`, this call should return
   with -1. */

#include "user.h"

/* data will be at the end of .bss section */
static char data[4096] __attribute__ ((section (".testEndmem,\"aw\",@nobits#")));

void main() {
    char* p = (char*)ROUND_UP(data + sizeof data - 1, 4096) - 1;
    *p = 'a';

    /* only the first byte of string p is valid */
    assert(open(p, O_CREATE) == -1, "part of the path is in a invalid memory space");
}
