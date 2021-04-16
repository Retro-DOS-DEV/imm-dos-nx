.intel_syntax noprefix
.code32
.global start

start:
  # install the interrupt handler on IRQ3
  mov eax, 0x40
  mov ebx, 0x03
  mov ecx, offset handler
  int 0x2b

  xor eax, eax

wait:
  jmp wait

handler:
  mov eax, 0xaa
  jmp handler
