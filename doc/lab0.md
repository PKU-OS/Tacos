# Lab 1: Appetizer

---

## Information

Name:

Email:

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

> With any comments that may help TAs to evaluate your work better, please leave them here

## Booting Tacos

> A1: Put the screenshot of Tacos running example here.

## Debugging

### First instruction

> B1: What is the first instruction that gets executed?

> B2: At which physical address is this instruction located?

### From ZSBL to SBI

> B3: Which address will the ZSBL jump to?

### SBI, kernel and argument passing

> B4: What's the value of the argument `hard_id` and `dtb`?

> B5: What's the value of `Domain0 Next Address`, `Domain0 Next Arg1`, `Domain0 Next Mode` and `Boot HART ID` in OpenSBI's output?

> B6: What's the relationship between the four output values and the two arguments?

### SBI interfaces

> B7: Inside `console_putchar`, Tacos uses `ecall` instruction to transfer control to SBI. What's the value of register `a6` and `a7` when executing that `ecall`?

## Kernel Monitor

> C1: Put the screenshot of your kernel monitor running example here. (It should show how your kernel shell respond to `whoami`, `exit`, and `other input`.)

> C2: Explain how you read and write to the console for the kernel monitor.
