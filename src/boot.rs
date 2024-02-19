// Entry point of the kernel. The kernel starts off at a low address (0x80200000),
// setting up the entry page table and then jumps to a high address space (0xFFFFFFC080000000).
// Then, after setting the boot stack, it enters the "main" function.
core::arch::global_asm! {r#"
    .section .text.entry
    .global _entry
    _entry:
        # load the physical address of the entry page table
        la t0, entry_pgtable

        # prepare satp's content
        srli t0, t0, 12
        li t1, 0x8 << 60
        or t0, t0, t1

        # enable paging
        sfence.vma zero, zero
        csrw satp, t0
        sfence.vma zero, zero

        # set up the stack
        ld t0, _relocated
        jr t0

    relocated:
        la sp, bootstack_top
        j main

    .globl _relocated
    _relocated:
        .8byte relocated

    .section .data
    .align 12
    .globl bootstack
    bootstack:
        .space 8 * 4096
    .globl bootstack_top
    bootstack_top:

    .align 12
    .globl entry_pgtable
    entry_pgtable:
        .zero 16
        .8byte 0x2000002F  # [PM_BASE, PM_BASE + 1GB) => [PM_BASE, PM_BASE + 1GB)
        .zero 2040
        .8byte 0x2000002F  # [VM_BASE, VM_BASE + 1GB) => [PM_BASE, PM_BASE + 1GB)
        .zero 2048
"#}
