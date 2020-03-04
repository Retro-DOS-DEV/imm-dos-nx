.intel_syntax noprefix
.code32

int_fault:
  lea esi, msg_system_fault
  call print_string_32
  hlt

.macro idt_entry addr
  .word offset \addr
  .word 0x8
  .byte 0
  .byte 0b10001110
  .word 0
.endm

idt_null_pointer:
  .word 0
  .long 0
idt_pointer:
  .word end_idt - idt - 1
  .long offset idt
idt:
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
  idt_entry int_fault
end_idt:

msg_system_fault: .asciz "System Fault! "
