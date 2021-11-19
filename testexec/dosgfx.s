.intel_syntax noprefix
.code16
.global start

start:
  # set up stack
  mov ax, 0x7000
  mov ss, ax
  mov sp, 0xfffe

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
  mov si, (offset char_table) + 16
  mov di, 320 * 10 + 10
  call put_char

  # wait for character before quitting
  mov ax, 0x0800
  int 0x21

  # exit without cleaning up
  xor ax, ax
  int 0x21

  jmp $  

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

  ret

char_table:
  .byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
  .byte 0x3c, 0x42, 0xa5, 0x81, 0xbd, 0x99, 0x42, 0x3c
  .byte 0x3c, 0x7e, 0xdb, 0xff, 0xc3, 0xe7, 0x7e, 0x3c
