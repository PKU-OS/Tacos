#ifndef __LIB_FSTAT_H
#define __LIB_FSTAT_H

#include "types.h"

#define T_DIR 1     // Directory
#define T_FILE 2    // File
#define T_DEVICE 3  // Device

typedef struct {
    uint ino;     // Inode number
    uint64 size;  // Size of file in bytes
} stat;

#endif
