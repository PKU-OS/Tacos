/** This program attempts to execute code at address 0, which is not mapped. */

#include "user.h"

typedef int (*volatile functionptr)(void);

void main(void) {
    functionptr fp = NULL;
    printf("Congratulations - you have successfully called NULL: %d", fp());
    panic("should have exited");
}
