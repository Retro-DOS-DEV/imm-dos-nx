# Stage 1 bootloader. Reads system values from BIOS, copies the kernel into
# memory, and enters protected mode before jumping to the kernel

.intel_syntax noprefix
.code16

.global start

start:
  xor ax, ax
  mov ds, ax
  lea si, msg_start
  call print_string_16

  # Enable A20 line
  in al, 0x92
  or al, 2
  out 0x92, al

  # set up GDT and null IDT
  cli
  lgdt [gdt_pointer]
  lidt [idt_null_pointer]

  # enter protected mode
  mov eax, cr0
  or eax, 1
  mov cr0, eax

  jmp 0x8:protected_mode

.code32
protected_mode:
  mov ax, 0x10
  mov dx, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax
  mov esp, 0x9fffc
  call disable_cursor
  lea esi, msg_set_up
  call print_string_32

  lidt [idt_pointer]

  mov ax, 0
  div dx

halt:
  cli
  hlt

.include "gdt.s"
.include "idt.s"
.include "print16.s"
.include "print32.s"

msg_start: .asciz "Booting...\r\n"
msg_set_up: .asciz "System is in 32 bit protected mode! "
