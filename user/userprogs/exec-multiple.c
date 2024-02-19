/** Executes and waits for multiple child processes. */

#include "user.h"

void main() {
    const char* args[] = {"child-simple", 0};
    assert(wait(exec(args[0], args)) == 81);
    assert(wait(exec(args[0], args)) == 81);
    assert(wait(exec(args[0], args)) == 81);
    assert(wait(exec(args[0], args)) == 81);
    assert(wait(exec(args[0], args)) == 81);
}
