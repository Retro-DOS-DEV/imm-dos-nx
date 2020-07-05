ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

SECTIONS
{
  /DISCARD/ :
  {
    *(.note*)
  }
}