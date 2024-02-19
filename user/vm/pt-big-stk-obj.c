/* Allocates and writes to a 64 kB object on the stack.
   This must succeed. */

#include "arc4.h"
#include "cksum.h"
#include "user.h"

void main() {
    char stk_obj[65536];
    arc4 arc4;

    arc4_init(&arc4, "foobar", 6);
    memset(stk_obj, 0, sizeof stk_obj);
    arc4_crypt(&arc4, stk_obj, sizeof stk_obj);

    assert(cksum(stk_obj, sizeof stk_obj) == 3256410166);
}
