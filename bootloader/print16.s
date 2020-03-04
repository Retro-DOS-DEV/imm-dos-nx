.intel_syntax noprefix
.code16

# Print a null-terminated string, located at si
print_string_16:
  cld
  push ax
  mov ah, 0x0e
  # print chars until zero is encountered
print_next_char_16:
  lodsb
  or al, al
  jz print_16_done
  int 0x10
  jmp print_next_char_16
print_16_done:
  pop ax
  ret
