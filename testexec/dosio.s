.intel_syntax noprefix
.code16
.global start

start:
  mov dx, offset msg_prompt
  mov ah, 0x09
  int 0x21

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

  mov dx, offset msg_done
  mov ah, 0x09
  int 0x21

  # read from file
  mov dx, offset read_file
  mov ah, 0x3d
  int 0x21
  jc failed

  mov bx, ax
  mov cx, 9
  mov dx, offset read_buffer
  mov ah, 0x3f
  int 0x21

  mov dx, offset msg_got
  mov ah, 0x09
  int 0x21

  mov dx, offset read_buffer
  int 0x21

  mov ah, 0x00
  int 0x21

  jmp $ # unreachable

failed:
  mov dx, offset msg_fail
  mov ah, 0x09
  int 0x21

  mov ah, 0x00
  int 0x21

  jmp $

msg_prompt: .ascii "Enter 5 characters: $"
msg_done: .ascii "DONE.\n$"
msg_fail: .ascii "Failed to read file$"
msg_got: .ascii "Got: $"
read_file: .asciz "INIT:\\test.txt"
read_buffer: .byte 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x24
