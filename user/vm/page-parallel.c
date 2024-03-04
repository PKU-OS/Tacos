/* Runs 4 child-linear processes at once. */

#include "user.h"

#define CHILD_CNT 4

void main() {
    int children[CHILD_CNT];
    int i;

    const char* args[] = {"child-linear", 0};
    for (i = 0; i < CHILD_CNT; i++) assert((children[i] = exec(args[0], args)) != -1);
    for (i = 0; i < CHILD_CNT; i++) assert(wait(children[i]) == 0, "wait for child %d", i);
}
