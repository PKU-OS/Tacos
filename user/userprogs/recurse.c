/** Executes itself recursively to the depth indicated by the
   first command-line argument. */

#include "user.h"

void main(int argc, char* argv[]) {
    assert(argc > 1);
    int n = atoi(argv[1]);

    if (n > 0) {
        char buf[10];
        itoa(buf, n - 1);

        int child;
        const char* argv[] = {"recurse", buf, 0};

        assert((child = exec(argv[0], argv)) != -1);
        assert(wait(child) == n - 1);
    }

    exit(n);
}
