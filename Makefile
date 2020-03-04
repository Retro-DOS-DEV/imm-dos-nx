diskimage := build/bootdisk.img
bootsector := build/boot/mbr.bin
bootsector_src := bootloader/mbr.s
bootsector_obj := build/boot/mbr.o
boot_includes := bootloader/
bootloader := build/boot/boot.bin
bootloader_src := bootloader/boot.s
bootloader_obj := build/boot/boot.o

.PHONY: all, clean

all: bootdisk

clean:
	@rm -r build

bootdisk: $(diskimage) $(bootsector) $(bootloader)
	@dd if=$(bootsector) of=$(diskimage) bs=450 count=1 seek=62 oflag=seek_bytes conv=notrunc
	@mcopy -D o -i $(diskimage) $(bootloader) ::BOOT.BIN

$(diskimage):
	@mkdir -p $(shell dirname $@)
	@mkfs.msdos -C $(diskimage) 1440

$(bootsector): $(bootsector_obj)
	@mkdir -p $(shell dirname $(bootsector))
	@ld -o $(bootsector) --oformat binary -e boot -m elf_i386 -Ttext 0x7c3e $(bootsector_obj)

$(bootsector_obj): $(bootsector_src)
	@mkdir -p $(shell dirname $(bootsector))
	@as --32 -march=i386 -o $(bootsector_obj) -I $(boot_includes) $(bootsector_src)

$(bootloader): $(bootloader_obj)
	@mkdir -p $(shell dirname $(bootloader))
	@ld -o $(bootloader) --oformat binary -e start -m elf_i386 -Ttext 0x5000 $(bootloader_obj)

$(bootloader_obj): $(bootloader_src) bootloader/*.s
	@mkdir -p $(shell dirname $(bootloader_obj))
	@as --32 -march=i386 -o $(bootloader_obj) -I $(boot_includes) $(bootloader_src)
