#ifndef __LIB_ARC4_H
#define __LIB_ARC4_H

#include "types.h"

/* Alleged RC4 algorithm encryption state. */
typedef struct arc4 {
    uint8 s[256];
    uint8 i, j;
} arc4;

void arc4_init(arc4*, const void*, size_t);
void arc4_crypt(arc4*, void*, size_t);

#endif
