# Lab 2: User Programs

---

## Information

Name:

Email:

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

> With any comments that may help TAs to evaluate your work better, please leave them here

## Argument Passing

#### DATA STRUCTURES

> A1: Copy here the **declaration** of each new or changed struct, enum type, and global variable. State the purpose of each within 30 words.

#### ALGORITHMS

> A2: Briefly describe how you implemented argument parsing. How do you arrange for the elements of argv[] to be in the right order? How do you avoid overflowing the stack page?

#### RATIONALE

> A3: In Tacos, the kernel reads the executable name and arguments from the command. In Unix-like systems, the shell does this work. Identify at least two advantages of the Unix approach.

## System Calls

#### DATA STRUCTURES

> B1: Copy here the **declaration** of each new or changed struct, enum type, and global variable. State the purpose of each within 30 words.

> B2: Describe how file descriptors are associated with open files. Are file descriptors unique within the entire OS or just within a single process?

#### ALGORITHMS

> B3: Describe your code for reading and writing user data from the kernel.

> B4: Suppose a system call causes a full page (4,096 bytes) of data to be copied from user space into the kernel. 
> What is the least and the greatest possible number of inspections of the page table 
> (e.g. calls to `Pagetable.get_pte(addr)` or other helper functions) that might result?
> What about for a system call that only copies 2 bytes of data?
> Is there room for improvement in these numbers, and how much?

> B5: Briefly describe your implementation of the "wait" system call and how it interacts with process termination.

> B6: Any access to user program memory at a user-specified address
> can fail due to a bad pointer value.  Such accesses must cause the
> process to be terminated.  System calls are fraught with such
> accesses, e.g. a "write" system call requires reading the system
> call number from the user stack, then each of the call's three
> arguments, then an arbitrary amount of user memory, and any of
> these can fail at any point.  This poses a design and
> error-handling problem: how do you best avoid obscuring the primary
> function of code in a morass of error-handling?  Furthermore, when
> an error is detected, how do you ensure that all temporarily
> allocated resources (locks, buffers, etc.) are freed?
> Have you used some features in Rust, to make these things easier than in C?
> In a few paragraphs, describe the strategy or strategies you adopted for
> managing these issues.  Give an example.

> B7: Briefly describe what will happen if loading the new executable fails. (e.g. the file does not exist, is in the wrong format, or some other error.)

#### SYNCHRONIZATION

> B8: Consider parent process P with child process C.  How do you
> ensure proper synchronization and avoid race conditions when P
> calls wait(C) before C exits?  After C exits?  How do you ensure
> that all resources are freed in each case?  How about when P
> terminates without waiting, before C exits?  After C exits?  Are
> there any special cases?

#### RATIONALE

> B9: Why did you choose to implement access to user memory from the
> kernel in the way that you did?

> B10: What advantages or disadvantages can you see to your design
> for file descriptors?

> B11: What is your tid_t to pid_t mapping. What advantages or disadvantages can you see to your design?
