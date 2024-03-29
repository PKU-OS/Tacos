#include <stdarg.h>

#include "user.h"

// wrapper so that it's OK if main() does not call exit().
void _main() {
    extern void main();
    main();
    exit(NORMAL_EXIT);
}

int strcmp(const char* p, const char* q) {
    while (*p && *p == *q) p++, q++;
    return (uchar)*p - (uchar)*q;
}

char* strcpy(char* s, const char* t) {
    char* os = s;

    while ((*s++ = *t++) != 0)
        ;
    return os;
}

int strlen(const char* s) {
    int n;

    for (n = 0; s[n]; n++)
        ;
    return n;
}

void* memset(void* dst, int c, uint n) {
    char* cdst = (char*)dst;
    int i;
    for (i = 0; i < n; i++) {
        cdst[i] = c;
    }
    return dst;
}

/* Find the first differing byte in the two blocks of SIZE bytes
   at A and B.  Returns a positive value if the byte in A is
   greater, a negative value if the byte in B is greater, or zero
   if blocks A and B are equal. */
int memcmp(const void* a_, const void* b_, uint64 size) {
    const uint8* a = a_;
    const uint8* b = b_;

    assert(a != NULL || size == 0);
    assert(b != NULL || size == 0);

    for (; size-- > 0; a++, b++)
        if (*a != *b) return *a > *b ? +1 : -1;
    return 0;
}

/* Copies SIZE bytes from SRC to DST, which must not overlap.
   Returns DST. */
void* memcpy(void* dst_, const void* src_, size_t size) {
    uint8* dst = dst_;
    const uint8* src = src_;

    assert(dst != NULL || size == 0);
    assert(src != NULL || size == 0);

    while (size-- > 0) *dst++ = *src++;

    return dst_;
}

static void reverse(char s[]) {
    int i, j;
    char c;

    for (i = 0, j = strlen(s) - 1; i < j; i++, j--) {
        c = s[i];
        s[i] = s[j];
        s[j] = c;
    }
}

// Convert n into a string
//
// Unsafe: please make sure the buffer is big enough to hold the string.
void itoa(char* buf, int n) {
    int i = 0, sign;

    if ((sign = n) < 0) n = -n;

    do {
        buf[i++] = n % 10 + '0';
    } while ((n /= 10) > 0);

    if (sign < 0) buf[i++] = '-';
    buf[i] = '\0';
    reverse(buf);
}

int atoi(const char* s) {
    int n = 0, sign = 1;

    while ((char)*s == ' ') s++;

    if (*s == '+') {
        s++;
    } else if (*s == '-') {
        sign = -1;
        s++;
    }

    while ('0' <= *s && *s <= '9') n = n * 10 + *s++ - '0';
    return sign * n;
}

static void compare_bytes(const void* read_data_, const void* expected_data_, size_t size,
                          size_t ofs, const char* file_name) {
    const uint8* read_data = read_data_;
    const uint8* expected_data = expected_data_;
    size_t i, j;

    if (!memcmp(read_data, expected_data, size)) return;

    for (i = 0; i < size; i++)
        if (read_data[i] != expected_data[i]) break;
    for (j = i + 1; j < size; j++)
        if (read_data[j] == expected_data[j]) break;

    panic(
        "%d bytes read starting at offset %d in \"%s\" differ "
        "from expected",
        j - i, ofs + i, file_name);
}

void check_file_handle(int fd, const char* file_name, const void* buf_, size_t size) {
    const char* buf = buf_;
    size_t ofs = 0;

    /* Warn about file of wrong size.  Don't fail yet because we
       may still be able to get more information by reading the
       file. */
    stat s;
    fstat(fd, &s);
    if (s.size != size)
        printf("size of %s (%d) differs from expected (%d)", file_name, s.size, size);

    /* Read the file block-by-block, comparing data as we go. */
    while (ofs < size) {
        char block[512];
        size_t block_size, ret_val;

        block_size = size - ofs;
        if (block_size > sizeof block) block_size = sizeof block;

        ret_val = read(fd, block, block_size);
        if (ret_val != block_size)
            panic("read of %d bytes at offset %d in \"%s\" returned %d", block_size, ofs,
                  file_name, ret_val);

        compare_bytes(block, buf + ofs, block_size, ofs, file_name);
        ofs += block_size;
    }

    /* Now fail due to wrong file size. */
    if (s.size != size)
        panic("size of %s (%d) differs from expected (%d)", file_name, s.size, size);

    printf("verified contents of \"%s\"", file_name);
}

void check_file(const char* file_name, const void* buf, size_t size) {
    int fd;

    assert((fd = open(file_name, 0)) > 2, "open \"%s\" for verification", file_name);
    check_file_handle(fd, file_name, buf, size);
    printf("close \"%s\"", file_name);
    close(fd);
}

inline uint64 r_sp() {
    uint64 x;
    asm volatile("mv %0, sp" : "=r"(x));
    return x;
}
