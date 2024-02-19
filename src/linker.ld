OUTPUT_ARCH(riscv)
ENTRY(_entry)

SECTIONS
{
    . = 0xFFFFFFC080200000;

	/* AT(...) gives the load address of this section, which tells
	   the boot loader where to load the kernel in physical memory */
    .text : AT(0x80200000) {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;

    .data : { *(.*data*) }

    . = ALIGN(16);
    .bss : {
        sbss = .;
        *(.*bss*)
        ebss = .;
    }

    . = ALIGN(4K);
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
