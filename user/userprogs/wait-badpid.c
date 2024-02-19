/** Waits for an invalid pid. This should fail and return with -1. */

#include "user.h"

void main() { assert(wait(0xbbaadd) == -1, "invalid pid"); }
