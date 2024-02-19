/** Child process run by exec-multiple, exec-one, wait-simple, and
   wait-twice tests.
   Just prints a single message and terminates. */

#include "user.h"

void main(int argc, char* argv[]) {
    assert(argv[argc] == NULL);
    assert(strcmp("child-simple", argv[0]) == 0);
    printf("run");

    exit(81);
}
