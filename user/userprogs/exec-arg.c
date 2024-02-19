/** Tests argument passing to child processes. */

#include "user.h"

void main() {
    const char* args[] = {"child-simple", "arg for child-simple", 0};
    assert(wait(exec(args[0], args)) == 81);
}
