.intel_syntax noprefix
.code16
.global start

start:
  mov bx, 0xbb

  mov ah, 0x02
  mov dl, 0x40
  int 0x21
  jmp $
