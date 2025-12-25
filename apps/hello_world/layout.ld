ENTRY(_start)

SECTIONS
{
    . = 0x0;
    .text : {
        *(.text._start)
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
}
