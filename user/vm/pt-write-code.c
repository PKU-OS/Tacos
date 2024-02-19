/* Try to write to the code segment.
   The process must be terminated with -1 exit code. */

#include "user.h"

void main() {
    *(int*)main = 0;
    panic("writing the code segment succeeded");
}
