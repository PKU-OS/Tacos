/** Child process run by close-by-child test.

   Attempts to close the file descriptor passed as the first
   command-line argument. This is invalid, because file
   descriptors are not inherited through `exec`. */

#include "user.h"

int main(int argc, char* argv[]) {
    assert(argc == 2);
    assert(atoi(argv[1]) > 2);
    assert(close(atoi(argv[1])) == -1);

    return 64;
}
