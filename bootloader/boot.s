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
  # argument order: dest, success message, error message, filename
  push 0x10
  push 0
  push offset msg_kernel_found
  push offset msg_kernel_not_found
  push offset filename_kernel
  call directory_entry_search
  pop dx
  pop dx
  pop dx

  pop edx
  # edx is the number of kernel bytes copied
  # round up to the next 4KiB barrier
  add edx, 0x1000
  and edx, 0xfffff000
  # set the destination for initfs
  add edx, 0x100000
  mov [initfs_start], edx
  push edx
  push offset msg_initfs_found
  push offset msg_initfs_not_found
  push offset filename_initfs
  mov bx, 0x7e00
  call directory_entry_search
  pop dx
  pop dx
  pop dx

  pop edx
  mov [initfs_size], edx

  jmp files_ready

directory_entry_search:
  # check if the directory entry is empty
  mov ah, [bx]
  cmp ah, 0
  je file_not_found
  # compare all letters of the filename
  mov cx, 11
  mov si, [esp + 2]
  push bx
check_filename_character:
  lodsb
  mov ah, [bx]
  cmp al, ah
  jnz check_next_entry
  inc bx
  loop check_filename_character
  jmp file_found
check_next_entry:
  pop bx
  add bx, 32
  jmp directory_entry_search
file_not_found:
  mov si, [esp + 4]
  call print_string_16
  jmp halt

file_found:
  pop bx
  mov si, [esp + 6]
  call print_string_16

  # copy the file to memory, via a lowmem buffer
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
copy_file_sector:
  call copy_sector_at_lba
  inc ax
  # if our buffer (0x8000 - 0xffff) is full, copy it to highmem via unreal mode
  cmp bx, 0xfe00
  jge copy_to_highmem
  add bx, 0x200
  loop copy_file_sector
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
  mov ebx, [esp + 10]
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
  je file_copied
  # otherwise, start copying to the buffer at 0x8000 again
  mov bx, 0x8000
  jmp copy_file_sector

file_copied:
  mov [esp + 14], edx
  pop edx
  pop dx
  ret

files_ready:
  # map memory
  call map_memory

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
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax
  mov esp, 0x9fffc
  call disable_cursor
  lea esi, msg_set_up
  call print_string_32

  lidt [idt_pointer]

  # read the ELF sections to find the end of the kernel in memory
  xor ecx, ecx
  mov cx, [0x100000 + 0x30]
  mov eax, [0x100000 + 0x20]
  xor edx, edx
read_section_header:
  mov ebx, [0x100000 + eax + 0x0c]
  add ebx, [0x100000 + eax + 0x14]
  cmp ebx, edx
  jb next_section_header
  mov edx, ebx

next_section_header:
  xor ebx, ebx
  mov bx, [0x100000 + 0x2e]
  add eax, ebx
  loop read_section_header

enter_kernel:
  # edx should be the furthest extent of any program section
  # move the stack pointer to the last four bytes of this section
  mov esp, edx
  sub esp, 0xc0000004

  push offset initfs_start
  push 0x00000000
  
  # read entrypoint from ELF header
  mov eax, [0x100000 + 0x18]
  sub eax, 0xc0000000
  jmp eax

symtab_not_found:
  call print_newline
  lea esi, msg_symtab_not_found
  call print_string_32

halt:
  cli
  hlt

.include "disk.s"
.include "gdt.s"
.include "idt.s"
.include "memory.s"
.include "print16.s"
.include "print32.s"
.include "unreal.s"

# BootStruct for passing values to the kernel
initfs_start: .long 0
initfs_size: .long 0

filename_kernel: .ascii "KERNEL  BIN"
filename_initfs: .ascii "INITFS  IMG"
msg_start: .asciz "Booting...\r\n"
msg_kernel_found: .asciz "Kernel found, loading into memory.\r\n"
msg_kernel_not_found: .asciz "Kernel not found!"
msg_initfs_found: .asciz "InitFS found, loading into memory.\r\n"
msg_initfs_not_found: .asciz "InitFS not found!"
msg_set_up: .asciz "System is in 32 bit protected mode! "
msg_symtab_not_found: .asciz "Kernel symbol table not found!"
