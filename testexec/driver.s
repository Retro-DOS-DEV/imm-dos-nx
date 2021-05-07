.intel_syntax noprefix
.code32
.global start

start:
  # install the interrupt handler on IRQ3
  mov eax, 0x40
  mov ebx, 0x03
  mov ecx, offset handler
  mov edx, offset stack_top
  int 0x2b

  mov eax, 0x0a
  mov ebx, 0x0b
  mov ecx, 0x0c
  mov edx, 0x0d
  mov esp, 0xfc

wait:
  # yield
  mov eax, 0x06
  int 0x2b
  jmp wait

handler:
  inc dword ptr [handler_hits]
  ret

.align 4
handler_hits:
.word 0

# create a 256-byte stack
.align 4
.skip 0x100, 0
stack_top:
