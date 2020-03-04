# Methods for loading sectors from disk using BIOS interrupts

.intel_syntax noprefix
.code16

# Constants for fetching bootsector metadata,
# assuming the MBR has been copied to 0x7c00
SECTORS_PER_CLUSTER = 0x7c0d
RESERVED_SECTORS = 0x7c0e
FAT_TABLE_COUNT = 0x7c10
ROOT_DIRECTORY_ENTRIES = 0x7c11
SECTORS_PER_FAT = 0x7c16
SECTORS_PER_TRACK = 0x7c18
DISK_HEAD_COUNT = 0x7c1a

# Initialize the disk metadata table
init_fat:
  # dl = disk number
  mov [disk_no], dl
  push bx
  push cx
  # bl will be a running count of sectors until each key section
  xor bh, bh
  mov bl, [SECTORS_PER_CLUSTER]
  mov [sectors_per_cluster], bx
  mov bl, [RESERVED_SECTORS]
  # skip each FAT table
  xor ch, ch
  mov cl, [FAT_TABLE_COUNT]
add_sectors_per_fat:
  add bl, [SECTORS_PER_FAT]
  loop add_sectors_per_fat

  mov [sector_of_root_dir], bx
  mov cx, [ROOT_DIRECTORY_ENTRIES]
  # multiply by 32 bytes per entry, divide by 512 bytes per sector
  shr cx, 4
  add bx, cx
  mov [sector_of_root_cluster], bx
  pop cx
  pop bx
  ret

# Copy a single sector from memory to disk
copy_sector_at_lba:
  # ax = lba representing the sector to copy
  # es:bx = sector copy destination
  push ax
  push cx
  push dx
  xor dx, dx
  mov cx, [SECTORS_PER_TRACK]
  div cx
  # ax = lba / sectors-per-track, dx = lba % sectors-per-track
  inc dl
  mov [lba_sector], dl
  xor dx, dx
  mov cx, [DISK_HEAD_COUNT]
  div cx
  # ax = track number, dx = head number
  mov [lba_track], al
  mov [lba_head], dl
  # try reading 3 times
  mov cx, 3
try_sector_read:
  push cx
  # call the BIOS disk read interrupt
  mov ah, 2
  mov al, 1
  mov ch, [lba_track]
  mov cl, [lba_sector]
  mov dh, [lba_head]
  mov dl, [disk_no]
  int 0x13
  pop cx
  jnc sector_copy_success
  loop try_sector_read
  # retries failed, should probably handle this
sector_copy_success:
  pop dx
  pop cx
  pop ax
  ret

# Disk metadata stored here
disk_no: .byte 0
sector_of_root_dir: .word 19
sector_of_root_cluster: .word 33
sectors_per_cluster: .word 1

# LBA deconstruction scratch space
lba_track: .byte 0
lba_sector: .byte 1
lba_head: .byte 0
