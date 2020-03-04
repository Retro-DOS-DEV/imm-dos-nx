# Code loaded into the MBR, used for the first stage of the bootloader
# Loads the first file off the disk into RAM, and jumps to its first byte

.intel_syntax noprefix
.code16

.global boot

boot:
  # dl is set to the current disk number
  # initialize the stack and segments
  mov sp, 0x7c00
  xor ax, ax
  mov ss, ax
  mov ds, ax
  mov es, ax
  # normalize cs
  push ax                     # will set segment to 0
  pushw offset boot_continue  # will set offset to boot_continue
  retf                        # far return pops both values

boot_continue:
  # read the root directory of the disk to 0x7e00
  call init_fat
  mov bx, 0x7e00
  mov ax, [sector_of_root_dir]
  call copy_sector_at_lba

  # confirm that the first file is called BOOT.BIN
  cld
  mov cx, 11
  lea si, expected_filename
  jmp check_filename
file_mismatch:
  lea si, msg_boot_not_found
  call print_string_16
  jmp halt
check_filename:
  lodsb
  mov ah, [bx]
  cmp al, ah
  jnz file_mismatch
  inc bx
  loop check_filename

  # read the starting sector of BOOT.BIN, as well as the byte size,
  # and load it at 0x5000
  lea si, msg_load_boot
  call print_string_16

  mov cx, [0x7e1d]  # high short of file size
  shr cx, 1         # in effect, dividing by 512
  inc cx            # capture the extra sector for the remaining bytes
  mov ax, [sector_of_root_cluster]
  mov bx, 0x5000
copy_boot_bin_sectors:
  call copy_sector_at_lba
  inc ax
  add bx, 0x200
  loop copy_boot_bin_sectors

  lea si, msg_start_boot
  call print_string_16

  jmp 0x5000

halt:
  cli
  hlt

.include "disk.s"
.include "print16.s"

expected_filename: .ascii "BOOT    BIN"
msg_boot_not_found: .asciz "BOOT.BIN not found"
msg_load_boot: .asciz "Loading Stage 1\r\n"
msg_start_boot: .asciz "Entering Stage 1\r\n"

.org 448 # account for extended FAT parameter block
.word 0xaa55
