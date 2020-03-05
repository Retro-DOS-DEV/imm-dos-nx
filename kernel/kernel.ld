ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

KERNEL_VADDR_OFFSET = 0;
KERNEL_OFFSET = 0x100000;

SECTIONS
{
  . = KERNEL_OFFSET;

  __kernel_start = .;

  . += SIZEOF_HEADERS;
  . = ALIGN(0x1000);

  __ro_physical_start = . - KERNEL_VADDR_OFFSET;
  .text : AT(ADDR(.text) - KERNEL_OFFSET)
  {
    *(.text*)

    . = ALIGN(0x1000);
  }

  .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET)
  {
    *(.rodata*)

    __ro_physical_end = . - KERNEL_VADDR_OFFSET;

    . = ALIGN(0x1000);
  }

  __rw_physical_start = . - KERNEL_VADDR_OFFSET;
  .data : AT(ADDR(.data) - KERNEL_OFFSET)
  {
    *(.data*)

    . = ALIGN(0x1000);
  }

  .bss : AT(ADDR(.bss) - KERNEL_OFFSET)
  {
    *(.bss*)

    . = ALIGN(0x1000);

    __stack_start = .;
    . += 4096;
    __stack_end = .;

    __rw_physical_end = . - KERNEL_VADDR_OFFSET;
  }

  __kernel_end = .;

  /DISCARD/ :
  {
    *(.note*)
  }
}