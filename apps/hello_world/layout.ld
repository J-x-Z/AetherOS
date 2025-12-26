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
        . = ALIGN(4096);
        _heap_start = .;
    }
}
