/** Executes itself recursively to the depth indicated by the
   first command-line argument. */

#include "user.h"

void main(int argc, char* argv[]) {
    assert(argc > 1);
    int n = atoi(argv[1]);
    assert(n > 0);

    char buf[10];
    itoa(buf, n - 1);

    int child;
    const char* child_argv[] = {"recurse", buf, 0};

    assert((child = exec(child_argv[0], child_argv)) != -1);
    assert(wait(child) == n - 1);
}
