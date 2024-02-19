/** Prints many command line arguments. */

#include "user.h"

void main(int argc, char* argv[]) {
    assert(argc == 23);
    assert(strcmp("args-many", argv[0]) == 0);

    char arg[] = "a";
    for (int i = 1; i < argc; ++i) {
        assert(strcmp(arg, argv[i]) == 0);
        ++arg[0];
    }
}
