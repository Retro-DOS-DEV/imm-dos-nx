.intel_syntax noprefix
.code16
.global start

start:
  xor cl, cl
  loop:
  mov ah, 0x01
  int 0x21
  mov dl, ' '
  mov ah, 0x02
  int 0x21
  inc cl
  cmp cl, 5
  jne loop

  mov dx, offset msg
  mov ah, 0x09
  int 0x21

  mov ah, 0x00
  int 0x21

  jmp $ # unreachable

msg: .ascii "DONE.$"
