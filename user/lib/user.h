#ifndef __LIB_USER_H
#define __LIB_USER_H

#include <stdarg.h>

#include "fcntl.h"
#include "fstat.h"
#include "types.h"

#define NULL ((void*)0)
#define ROUND_UP(p, align) (((uint64)p + (align)-1) / (align) * (align))
#define ROUND_DOWN(p, align) ((uint64)p / (align) * (align))
#define PANIC_EXIT 12345
#define NORMAL_EXIT 0

#define panic(fmt, args...)                                                       \
    do {                                                                          \
        fprintf(2, "panicked at '" fmt "', %s:%d\n", ##args, __FILE__, __LINE__); \
        exit(PANIC_EXIT);                                                         \
    } while (0)

#define assert(condition, args...)                        \
    if (condition) {                                      \
    } else {                                              \
        panic("assertion failed: `" #condition "`" args); \
    }

// system calls
void halt(void);
void exit(int status);
int exec(const char* pathname, const char* argv[]);
int wait(int pid);
int remove(const char* pathname);
int open(const char* pathname, int flags);
int read(int fd, void* buffer, uint size);
int write(int fd, const void* buffer, uint size);
void seek(int fd, uint position);
int tell(int fd);
int close(int fd);
int fstat(int fd, stat* buf);
int mmap(int fd, void* addr);
void munmap(int mapid);
int chdir(const char* dir);
int mkdir(const char* dir);

// ulib.c
void fprintf(int fd, const char* fmt, ...);
void printf(const char* fmt, ...);
int strcmp(const char*, const char*);
char* strcpy(char*, const char*);
int strlen(const char*);
void itoa(char*, int);
int atoi(const char*);
void* memset(void*, int, uint);
int memcmp(const void*, const void*, uint64);
void* memcpy(void*, const void*, size_t);
void check_file(const char*, const void* buf, size_t);
void check_file_handle(int fd, const char* file_name, const void* buf_, size_t size);
uint64 r_sp();

#endif
