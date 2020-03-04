.intel_syntax noprefix
.code16

gdt_pointer:
  .word end_gdt - gdt - 1 # limit
  .long offset gdt
gdt:
  # null entry
  .quad 0
  # code entry
  .word 0xffff      # limit low
  .word 0           # base low
  .byte 0           # base mid
  .byte 0b10011010  # access
  .byte 0b11001111  # flags, limit high
  .byte 0           # base high
  # data entry
  .word 0xffff      # limit low
  .word 0           # base low
  .byte 0           # base mid
  .byte 0b10010010  # access
  .byte 0b11001111  # flags, limit high
  .byte 0           # base high
end_gdt:
