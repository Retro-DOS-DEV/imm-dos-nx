.intel_syntax noprefix
.code32

.global irq_0, irq_1, irq_3, irq_4, irq_5, irq_6, irq_7, irq_8, irq_9, irq_10, irq_11, irq_12, irq_13, irq_14, irq_15

irq_0:
  pushd 0
  jmp irq_core

irq_1:
  pushd 1
  jmp irq_core

irq_3:
  pushd 3
  jmp irq_core

irq_4:
  pushd 4
  jmp irq_core

irq_5:
  pushd 5
  jmp irq_core

irq_6:
  pushd 6
  jmp irq_core

irq_7:
  pushd 7
  jmp irq_core

irq_8:
  pushd 8
  jmp irq_core

irq_9:
  pushd 9
  jmp irq_core

irq_10:
  pushd 10
  jmp irq_core

irq_11:
  pushd 11
  jmp irq_core

irq_12:
  pushd 12
  jmp irq_core

irq_13:
  pushd 13
  jmp irq_core

irq_14:
  pushd 14
  jmp irq_core

irq_15:
  pushd 15
  jmp irq_core

.text
irq_core:
  push eax
  push ecx
  push edx
  push ebx
  push ebp
  push esi
  push edi
  mov ebx, esp
  push ebx

  call _irq_inner

  add esp, 4
  pop edi
  pop esi
  pop ebp
  pop ebx
  pop edx
  pop ecx
  pop eax
  add esp, 4 # clear the irq number
irq_loop:
  jmp irq_loop
  iretd
