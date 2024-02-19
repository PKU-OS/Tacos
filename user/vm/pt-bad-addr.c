/* Accesses a bad address.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() { panic("bad addr read as %d", *(int*)0x04000000); }
