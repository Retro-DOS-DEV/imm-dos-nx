.intel_syntax noprefix
.code32
.global start

start:

read:
  mov ecx, offset buffer
  mov ebx, 0  # read from stdin
  mov edx, 1
  mov eax, 0x12
  int 0x2b

  mov edx, eax
  mov ebx, 1  # write to stdout
  mov eax, 0x13
  int 0x2b

  mov eax, 0x06
  int 0x2b
  jmp read

buffer: .byte 0
