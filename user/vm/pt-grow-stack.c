/* Demonstrate that the stack can grow.
   This must succeed. */

#include "arc4.h"
#include "cksum.h"
#include "user.h"

void main() {
    char stack_obj[4096];
    arc4 arc4;

    arc4_init(&arc4, "foobar", 6);
    memset(stack_obj, 0, sizeof stack_obj);
    arc4_crypt(&arc4, stack_obj, sizeof stack_obj);

    assert(cksum(stack_obj, sizeof stack_obj) == 3424492700);
}
