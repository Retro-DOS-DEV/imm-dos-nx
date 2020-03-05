.intel_syntax noprefix
.code16

# enter unreal mode, so we can address >1MB of data
# while still using real mode BIOS interrupts
unreal:
  push ds
  push ax
  push bx
  # enter protected mode
  lgdt [unreal_gdt_pointer]
  mov eax, cr0
  or al, 1
  mov cr0, eax
  jmp unreal_protected
unreal_protected:
  # set ds to the gdt entry
  mov bx, 0x08
  mov ds, bx
  # exit real mode by removing the protected mode bit
  and al, 0xfe
  mov cr0, eax
  pop bx
  pop ax
  # restore the ds selector, though the cached access via the GDT remains
  pop ds
  sti
  ret

unreal_gdt_pointer:
  .word end_unreal_gdt - unreal_gdt - 1
  .long offset unreal_gdt
unreal_gdt:
  # null entry
  .quad 0
  # single data entry
  .word 0xffff      # limit low
  .word 0           # base low
  .byte 0           # base mid
  .byte 0b10010010  # access
  .byte 0b11001111  # flags, limit high
  .byte 0           # base high
end_unreal_gdt:
