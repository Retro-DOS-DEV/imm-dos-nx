.intel_syntax noprefix
.code16

# Use BIOS interrupts to generate a map of all available memory that we can
# pass to the kernel.
# The memory map is stored as 0x1000:
# We store the number of entries, followed by the first entry at 0x1004
memory_map_length = 0x1000
map_memory:
  push eax
  push ebx
  push ecx
  push edx
  push edi
  push esi

  # use int 0x15, eax=0xe820 to detect memory
  mov di, 0x1004
  xor esi, esi
  xor ebx, ebx
  mov edx, 0x534d4150
map_memory_loop:
   mov eax, 0xe820
   mov ecx, 24
   int 0x15
   # if carry is set, map is completed
   jc map_memory_finished
   # if ebx is zero, map is completed
   cmp ebx, 0
   je map_memory_finished
   # eax should now equal edx
   cmp eax, edx
   jne map_memory_finished

   # increment values, and loop
   add di, 24
   inc esi
   # arbitrarily cap at 170 entries, which would fill up to 0x2000
   cmp esi, 170
   jb map_memory_loop

  map_memory_finished:
    mov [memory_map_length], esi

    pop esi
    pop edi
    pop edx
    pop ecx
    pop ebx
    pop eax
    ret
