.intel_syntax noprefix
.code32

# Text printing methods intended to only be used in 32-bit mode

# column coordinate of the cursor
print_cursor_col: .byte 0

# hide the blinking cursor
disable_cursor:
  push eax
  push edx
  mov al, 0x0a
  mov dx, 0x3d4
  out dx, al
  mov al, 0x20
  mov dx, 0x3d5
  out dx, al
  pop edx
  pop eax
  ret

# scroll to the next line and move the cursor to position 0
print_newline:
  call scroll_window
  movb [print_cursor_col], 0
  ret

# print a zero-terminated string, located at esi
print_string_32:
  cld
  push eax
  push edx
  mov ah, 0x0b
  # print chars until zero is encountered
print_next_char_32:
  lodsb
  or al, al
  jz print_32_done
  xor edx, edx
  mov dl, print_cursor_col
  add edx, 0xb8000 + 160 * 24
  mov [edx], ax
  # check if we reached the end of the row
  # if so, scroll to the next line and move the cursor to 0
  cmp edx, 0xb8000 + 160 * 24 + 158
  jl increment_cursor
  call scroll_window
  movb [print_cursor_col], -2
increment_cursor:
  addb [print_cursor_col], 2
  jmp print_next_char_32
print_32_done:
  pop edx
  pop eax
  ret

# scroll the text buffer upwards by one row
scroll_window:
  push eax
  push ebx
  push ecx
  mov ecx, 40 * 24
  # copy the data from one row to the row above, four bytes at a time
move_text_video_bytes:
  mov eax, 0xb8000 + 160 * 24 + 160
  lea ebx, [ecx * 4]
  sub eax, ebx
  mov ebx, [eax]
  sub eax, 160
  mov [eax], ebx
  loop move_text_video_bytes
  # fill the last row with spaces
  mov ecx, 40
clear_last_row:
  lea eax, [ecx * 4 + (0xb8000 + 160 * 24 - 4)]
  mov dword ptr [eax], 0x0b200b20
  loop clear_last_row
  pop ecx
  pop ebx
  pop eax
  ret
