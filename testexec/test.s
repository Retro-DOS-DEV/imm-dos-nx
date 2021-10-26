.intel_syntax noprefix
.code32
.global start

start:
  # find heap start
  mov eax, 0x04
  mov ebx, 1
  mov ecx, 0
  int 0x2b
  mov edi, eax

  # make space on the heap
  mov eax, 0x04
  mov ebx, 0
  mov ecx, edi
  add ecx, 0x3000
  int 0x2b
  movb [edi + 0x2200], 0xfa

  # shrink the heap a bit
  mov eax, 0x04
  mov ebx, 0
  mov ecx, edi
  add ecx, 0x1000
  int 0x2b

  # attempt a write to heap space
  movb [edi + 4], 0xfc

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

  mov eax, 0
  mov ebx, 1
  int 0x2b
end:
  jmp end

file_path: .ascii "DEV:\\COM1"
.equ file_path_len, . - file_path
.align 4
file_path_ptr: .long offset file_path, file_path_len

message: .ascii " *HI FROM EXEC* "
.equ message_len, . - message
