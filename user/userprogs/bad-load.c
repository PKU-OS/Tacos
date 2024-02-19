/** Tries to dereference an invalid pointer, which will result
   in a segmentation fault. */

#include "user.h"

void main() {
    printf("Congratulations - you have successfully dereferenced NULL: %d\n", *(int*)NULL);
    panic("should have exited");
}
