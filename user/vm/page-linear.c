/* Encrypts, then decrypts, 2 MB of memory and verifies that the
   values are as they should be. */

#include "arc4.h"
#include "user.h"

#define SIZE (2 * 1024 * 1024)

static char buf[SIZE];

void main() {
    arc4 arc4;
    size_t i;

    /* Initialize to 0x5a. */
    memset(buf, 0x5a, sizeof buf);

    /* Check that it's all 0x5a. */
    for (i = 0; i < SIZE; i++)
        if (buf[i] != 0x5a) panic("byte %d != 0x5a", i);

    /* Encrypt zeros. */
    arc4_init(&arc4, "foobar", 6);
    arc4_crypt(&arc4, buf, SIZE);

    /* Decrypt back to zeros. */
    arc4_init(&arc4, "foobar", 6);
    arc4_crypt(&arc4, buf, SIZE);

    /* Check that it's all 0x5a. */
    for (i = 0; i < SIZE; i++)
        if (buf[i] != 0x5a) panic("byte %d != 0x5a", i);
}
