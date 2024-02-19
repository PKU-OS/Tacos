/* Child process run by rox-child and rox-multichild tests.
   Opens and tries to write to its own executable, verifying that
   that is disallowed.
   Then recursively executes itself to the depth indicated by the
   first command-line argument. */

#include "user.h"

static void try_write(void) {
    int fd;
    char buffer[19];

    assert((fd = open("child-rox", O_WRONLY)) > 2);
    assert(write(fd, buffer, sizeof buffer) == -1, "\"child-rox\" is not writable");

    close(fd);
}

void main(int argc, char* argv[]) {
    try_write();

    assert(argc > 1);

    int n = atoi(argv[1]);

    if (n > 1) {
        int child;
        char buf[10];
        itoa(buf, n - 1);

        const char* args[] = {"child-rox", buf, 0};
        assert((child = exec(args[0], args)) >= 0);
        assert(wait(child) == 12);
    }

    try_write();

    exit(12);
}
