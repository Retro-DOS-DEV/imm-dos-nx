.intel_syntax noprefix
.code16
.global start

start:
  # wait for character before doing anything
  mov ax, 0x0800
  int 0x21

  # enter mode 13h
  mov ax, 0x0013
  int 0x10

  mov ax, ds
  mov dx, 0xa000
  mov ds, dx
  mov bx, 200 * 320
  clear_pixel:
  movw [bx], 0x0f0f
  sub bx, 2
  jnz clear_pixel
  movw [bx], 0x0f0f

  mov ds, ax
  mov si, offset char_table
  mov di, 320 * 10 + 10

# ds:si = location of an 8-byte character
# di    = location to place the character in the video buffer
put_char:
  mov bx, si
  add bx, 8

  put_char_line:
  mov dh, 0x08
  mov ds, ax
  mov dl, [si]
  mov cx, 0xa000
  mov ds, cx

  put_char_pixel:
  shl dl
  jnc no_pixel
  movb [di], 0x00
  no_pixel:
  inc di
  dec dh
  jnz put_char_pixel
  inc si
  add di, 320 - 8
  cmp si, bx
  jl put_char_line

  # exit without cleaning up
  xor ax, ax
  int 0x21

char_table:
  .byte 0x3c, 0x42, 0xa5, 0x81, 0xa5, 0x99, 0x42, 0x3c
