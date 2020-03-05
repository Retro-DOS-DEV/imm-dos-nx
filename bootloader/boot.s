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

  mov ax, 0
  div dx

halt:
  cli
  hlt

.include "disk.s"
.include "gdt.s"
.include "idt.s"
.include "print16.s"
.include "print32.s"

filename_kernel: .ascii "KERNEL  BIN"
msg_start: .asciz "Booting...\r\n"
msg_kernel_found: .asciz "Kernel found, loading into memory.\r\n"
msg_kernel_not_found: .asciz "Kernel not found!"
msg_set_up: .asciz "System is in 32 bit protected mode! "
