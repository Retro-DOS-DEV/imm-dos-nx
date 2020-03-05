# Stage 1 bootloader. Reads system values from BIOS, copies the kernel into
# memory, and enters protected mode before jumping to the kernel

.intel_syntax noprefix
.code16

.global start

start:
  xor ax, ax
  mov ds, ax
  lea si, msg_start
  call print_string_16

  # Enable A20 line
  in al, 0x92
  or al, 2
  out 0x92, al

  # read the directory entries from disk
  # we assume we still have the MBR at 0x7c00
  call init_fat
  mov bx, 0x7e00
  mov ax, [sector_of_root_dir]
  call copy_sector_at_lba

  # find the kernel
  cld
directory_entry_search:
  # check if the directory entry is empty
  mov ah, [bx]
  cmp ah, 0
  je kernel_not_found
  # compare all letters of the filename
  mov cx, 11
  lea si, filename_kernel
  push bx
check_filename_character:
  lodsb
  mov ah, [bx]
  cmp al, ah
  jnz check_next_entry
  inc bx
  loop check_filename_character
  jmp kernel_found
check_next_entry:
  pop bx
  add bx, 32
  jmp directory_entry_search
kernel_not_found:
  lea si, msg_kernel_not_found
  call print_string_16
  jmp halt

kernel_found:
  pop bx
  lea si, msg_kernel_found
  call print_string_16

  # copy the kernel to memory, via a lowmem buffer
  # determine the size, and start loading data at 0x8000
  # this is the same trick from the MBR loader, we convert bytes into sectors
  mov cx, [bx + 0x1d]
  shr cx, 1
  inc cx
  # get the starting cluster, and use it to find the starting sector
  mov ax, [bx + 0x1a]
  sub ax, 2
  mov dx, [sectors_per_cluster]
  imul dx
  add ax, [sector_of_root_cluster]
  mov bx, 0x8000
  mov edx, 0
  push cx   # sectors left to read
  push edx  # bytes copied so far
copy_kernel_sector:
  call copy_sector_at_lba
  inc ax
  # if our buffer (0x8000 - 0xffff) is full, copy it to highmem via unreal mode
  cmp bx, 0xfe00
  jge copy_to_highmem
  add bx, 0x200
  loop copy_kernel_sector
  inc cx
copy_to_highmem:
  dec cx
  # set remaining sectors to copy in bx
  mov bx, cx
  # set bytes already copied in edx
  pop edx
  # set sectors left to read in cx
  pop cx
  # subtract sectors remaining from sectors left to read, to determine how many
  # we covered in the last loop
  sub cx, bx
  # put sectors left to read back on the stack
  push bx
  # convert ecx from # of sectors to # of words, by multiplying by 256
  and ecx, 0xffff
  shl ecx, 8
  # copy $ecx words from lowmem to highmem
  mov esi, 0x8000
  mov ebx, 0x100000
  add ebx, edx
  push ax
  # unlock "unreal" mode to make >1MB addressable in real mode
  call unreal
copy_to_highmem_loop:
  lodsw
  mov [ebx], ax
  add ebx, 2
  add edx, 2
  loop copy_to_highmem_loop

  # return ax to the next sector we need to read
  pop ax
  # read the sectors left to read into cx, without popping
  mov bx, sp
  mov cx, [bx]
  # put bytes copied back on the stack
  push edx

  # if cx is zero, there are no sectors left to read
  cmp cx, 0
  je kernel_copied
  # otherwise, start copying to the buffer at 0x8000 again
  mov bx, 0x8000
  jmp copy_kernel_sector

kernel_copied:
  # set up GDT and null IDT
  cli
  lgdt [gdt_pointer]
  lidt [idt_null_pointer]

  # enter protected mode
  mov eax, cr0
  or eax, 1
  mov cr0, eax

  jmp 0x8:protected_mode

.code32
protected_mode:
  mov ax, 0x10
  mov dx, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax
  mov esp, 0x9fffc
  call disable_cursor
  lea esi, msg_set_up
  call print_string_32

  lidt [idt_pointer]

  # read entrypoint from ELF header
  mov eax, [0x100000 + 0x18]
  jmp eax

halt:
  cli
  hlt

.include "disk.s"
.include "gdt.s"
.include "idt.s"
.include "print16.s"
.include "print32.s"
.include "unreal.s"

filename_kernel: .ascii "KERNEL  BIN"
msg_start: .asciz "Booting...\r\n"
msg_kernel_found: .asciz "Kernel found, loading into memory.\r\n"
msg_kernel_not_found: .asciz "Kernel not found!"
msg_set_up: .asciz "System is in 32 bit protected mode! "
