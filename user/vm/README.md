Functionality of virtual memory subsystem:
- Test stack growth.
3	pt-grow-stack
3	pt-grow-stk-sc
3	pt-big-stk-obj
[x]	pt-grow-pusha (no `pusha` in risc-v)

- Test paging behavior.
3	page-linear
3	page-parallel
3	page-shuffle
4	page-merge-seq
4	page-merge-par
4	page-merge-mm
4	page-merge-stk

- Test "mmap" system call.
2	mmap-read
2	mmap-write
2	mmap-shuffle

2	mmap-twice

2	mmap-unmap
1	mmap-exit

3	mmap-clean

2	mmap-close
2	mmap-remove

Robustness of virtual memory subsystem:
- Test robustness of page table support.
2	pt-bad-addr
3	pt-bad-read
2	pt-write-code
3	pt-write-code2
4	pt-grow-bad

- Test robustness of "mmap" system call.
1	mmap-bad-fd
1	mmap-inherit
1	mmap-null
1	mmap-zero

2	mmap-misalign

2	mmap-over-code
2	mmap-over-data
2	mmap-over-stk
2	mmap-overlap

[ ] Choose an exit code for different errors
