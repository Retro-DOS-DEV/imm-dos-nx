.intel_syntax noprefix
.code32
.global start

start:
  # Open TTY0
  mov eax, 0x10
  lea ebx, file_path_ptr
  int 0x2b
  mov ebx, eax # file handle

read:
  mov ecx, offset buffer
  mov edx, 1
  mov eax, 0x12
  int 0x2b

  mov edx, eax
  mov eax, 0x13
  int 0x2b

  mov eax, 0x06
  int 0x2b
  jmp read

buffer: .byte 0

file_path: .ascii "DEV:\\TTY0"
.equ file_path_len, . - file_path
.align 4
file_path_ptr: .long offset file_path, file_path_len
