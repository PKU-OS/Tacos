/* Shuffles a 128 kB data buffer 10 times, printing the checksum
   after each time. */

#include "cksum.h"
#include "random.h"
#include "user.h"

#define SIZE (128 * 1024)

static char buf[SIZE];

uint32 _init = 3115322833;
uint32 _shuffle[] = {1691062564, 1973575879, 1647619479, 96566261,  3885786467,
                     3022003332, 3614934266, 2704001777, 735775156, 1864109763};

void main() {
    size_t i;

    /* Initialize. */
    for (i = 0; i < sizeof buf; i++) buf[i] = i * 257;
    assert(cksum(buf, sizeof buf) == _init);

    /* Shuffle repeatedly. */
    for (i = 0; i < 10; i++) {
        shuffle(buf, sizeof buf, 1);
        assert(cksum(buf, sizeof buf) == _shuffle[i]);
    }
}
