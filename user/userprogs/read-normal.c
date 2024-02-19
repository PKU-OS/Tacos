/** Try reading a file in the most normal way. */

#include "sample.inc"
#include "user.h"

void main() {
    check_file("sample.txt", sample, sizeof(sample) - 1);
}
