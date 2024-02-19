/** Wait for a subprocess to finish, twice.
   The first call must wait in the usual way and return the exit code.
   The second wait call must return -1 immediately. */

#include "user.h"

void main() {
    const char* argv[] = {"child-simple", 0};
    int pid = exec(argv[0], argv);
    assert(wait(pid) == 81);
    assert(wait(pid) == -1);
}
