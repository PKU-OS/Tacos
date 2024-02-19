/** Wait for a process that will be killed for bad behavior.
   No matter the child process was killed or exited gracefully,
   the first wait should succeed and the second one should fail. */

#include "user.h"

void main() {
    int pid;
    const char* args[] = {"child-bad", 0};

    assert((pid = exec(args[0], args)) >= 0);
    assert(wait(pid) == -1, "child-bad should exit with a non-zero code");
}
