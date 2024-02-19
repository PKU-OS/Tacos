/** Wait for a subprocess to finish. */

#include "user.h"

void main() {
    const char* argv[] = {"child-simple", 0};
    assert(wait(exec(argv[0], argv)) == 81);
}
