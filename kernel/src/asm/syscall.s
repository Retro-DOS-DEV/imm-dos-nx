.intel_syntax noprefix
.code32

.global syscall_handler

.text
syscall_handler:
  push eax
  push ecx
  push edx
  push ebx
  push ebp
  push esi
  push edi
  mov ebx, esp
  push ebx
  add ebx, 7 * 4
  push ebx

  call _syscall_inner

  add esp, 8
  pop edi
  pop esi
  pop ebp
  pop ebx
  pop edx
  pop ecx
  pop eax
  iretd
