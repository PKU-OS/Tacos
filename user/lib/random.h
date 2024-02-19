#ifndef __LIB_RANDOM_H
#define __LIB_RANDOM_H

#include "types.h"

void random_init(unsigned seed);
void random_bytes(void*, size_t);
unsigned long random_ulong(void);
void shuffle(void*, size_t cnt, size_t size);

#endif
