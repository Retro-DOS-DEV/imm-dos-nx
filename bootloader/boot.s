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

halt:
  cli
  hlt

.include "print16.s"

msg_start: .asciz "Booting...\r\n"
