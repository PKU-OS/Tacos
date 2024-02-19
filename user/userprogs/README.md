# LAB2 Syscall Tests

## Functionality of system calls

- Test argument passing on command line.
    - args-none
    - args-many

- Test "open" system call.
    - open-create
    - open-rdwr
    - open-trunc
    - open-many

- Test "read" system call.
    - read-normal
    - read-zero

- Test "write" system call.
    - write-normal
    - write-zero

- Test "close" system call.
    - close-normal

- Test "exec" system call.
    - exec-once
    - exec-multiple
    - exec-arg

- Test "wait" system call.
    - wait-simple
    - wait-twice

- Test "exit" system call.
    - exit

- Test "halt" system call.
    - halt

- Test recursive execution of user programs.
    - multi-recurse

- Test read-only executable feature.
    - rox-simple
    - rox-child
    - rox-multichild

## Robustness of system calls

- Test robustness of file descriptor handling.
    - close-stdio
	- close-badfd
	- close-twice
    - close-by-child
	- read-badfd
	- read-stdout
	- write-badfd
	- write-stdin

- Test robustness of buffer copying across page boundaries.
    - boundary-normal
    - boundary-bad

- Test handling of null pointer and empty strings.
    - open-invalid

- Test robustness of system call implementation and pointer handling.
    - sc-bad-sp
    - sc-bad-args

- Test robustness of "exec" and "wait" system calls.
    - exec-invalid
    - wait-bad-pid
    - wait-killed

- Test robustness of exception handling.
    - bad-load
    - bad-store
    - bad-jump
    - bad-load2
    - bad-store2
    - bad-jump2
