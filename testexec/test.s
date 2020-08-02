.intel_syntax noprefix
.code32
.global start

start:
  # make space on the heap
  mov eax, 0x04
  mov ebx, 0
  mov ecx, 0x1000
  int 0x2b
  # attempt a write to heap space
  movb [0x1004], 0xfc

  mov eax, 0x10
  lea ebx, file_path_ptr
  int 0x2b
  mov ebx, eax # file handle
  mov ecx, 0xffffffff
  mov eax, 0x1d
  int 0x2b

  mov ebx, eax
  lea ecx, message
  lea edx, message_len
  mov eax, 0x13
  int 0x2b
  mov eax, 0x06

end:
  int 0x2b
  jmp end

file_path: .ascii "DEV:\\COM1"
.equ file_path_len, . - file_path
.align 4
file_path_ptr: .long offset file_path, file_path_len

message: .ascii " *HI FROM EXEC* "
.equ message_len, . - message
