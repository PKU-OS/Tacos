/* RC4-based pseudo-random number generator (PRNG).

   RC4 is a stream cipher.  We're not using it here for its
   cryptographic properties, but because it is easy to implement
   and its output is plenty random for non-cryptographic
   purposes.

   See http://en.wikipedia.org/wiki/RC4_(cipher) for information
   on RC4.*/

#include "random.h"

/* RC4 state. */
static uint8 s[256];   /* S[]. */
static uint8 s_i, s_j; /* i, j. */

/* Already initialized? */
static int inited;

/* Swaps some bytes pointed to by A and B. */
static void swap_bytes(void* a_, void* b_, size_t size) {
    uint8* a = a_;
    uint8* b = b_;
    size_t i;

    for (i = 0; i < size; i++) {
        uint8 t = a[i];
        a[i] = b[i];
        b[i] = t;
    }
}

/* Initializes or reinitializes the PRNG with the given SEED. */
void random_init(unsigned seed) {
    uint8* seedp = (uint8*)&seed;
    int i;
    uint8 j;

    if (inited) return;

    for (i = 0; i < 256; i++) s[i] = i;
    for (i = j = 0; i < 256; i++) {
        j += s[i] + seedp[i % sizeof seed];
        swap_bytes(s + i, s + j, 1);
    }

    s_i = s_j = 0;
    inited = 1;
}

/* Writes SIZE random bytes into BUF. */
void random_bytes(void* buf_, size_t size) {
    uint8* buf;

    if (!inited) random_init(0);

    for (buf = buf_; size-- > 0; buf++) {
        uint8 s_k;

        s_i++;
        s_j += s[s_i];
        swap_bytes(s + s_i, s + s_j, 1);

        s_k = s[s_i] + s[s_j];
        *buf = s[s_k];
    }
}

/* Returns a pseudo-random unsigned long.
   Use random_ulong() % n to obtain a random number in the range
   0...n (exclusive). */
uint64 random_ulong(void) {
    uint64 ul;
    random_bytes(&ul, sizeof ul);
    return ul;
}

void shuffle(void* buf_, size_t cnt, size_t size) {
    uint8* buf = buf_;
    size_t i;

    for (i = 0; i < cnt; i++) {
        size_t j = i + random_ulong() % (cnt - i);
        swap_bytes(buf + i * size, buf + j * size, size);
    }
}
