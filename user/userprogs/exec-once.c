/** Executes and waits for a single child process. */

#include "user.h"

void main() {
    const char* args[] = {"child-simple", 0};
    assert(wait(exec(args[0], args)) == 81);
}
