diskimage := build/bootdisk.img
bootsector := build/boot/mbr.bin
bootsector_src := bootloader/mbr.s
bootsector_obj := build/boot/mbr.o
boot_includes := bootloader/
bootloader := build/boot/boot.bin
bootloader_src := bootloader/boot.s
bootloader_obj := build/boot/boot.o

kernel := build/kernel.bin
kernel_testing := build/kernel_testing.bin
libkernel := build/libkernel.a
libkernel_testing := build/libkernel_testing.a
kernel_linker := kernel/kernel.ld
kernel_deps := kernel/src/* kernel/src/*/* kernel/src/*/*/*

initfs := build/initfs.img

native_linker_elf := dos-native-elf.ld
testexec := initfs/test.bin
testcom := initfs/test.com
testdriver := initfs/driver.bin
testecho := initfs/echo.bin
dosio := initfs/dosio.com
elftest := initfs/elftest.elf
command := initfs/command.elf

.PHONY: all, clean, test

all: bootdisk

clean:
	@rm -r build

test: bootdisk_testing
	@qemu-system-i386 -m 8M -fda $(diskimage) -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04; \
	EXIT_CODE=$$?; \
	if [ $$EXIT_CODE = "7" ]; then exit 1; fi

bootdisk: $(diskimage) $(bootsector) $(bootloader) $(kernel) $(initfs)
	@dd if=$(bootsector) of=$(diskimage) bs=450 count=1 seek=62 oflag=seek_bytes conv=notrunc
	@mcopy -D o -i $(diskimage) $(bootloader) ::BOOT.BIN
	@mcopy -D o -i $(diskimage) $(kernel) ::KERNEL.BIN
	@mcopy -D o -i $(diskimage) $(initfs) ::INITFS.IMG

bootdisk_testing: $(diskimage) $(bootsector) $(bootloader) $(kernel_testing)
	@dd if=$(bootsector) of=$(diskimage) bs=450 count=1 seek=62 oflag=seek_bytes conv=notrunc
	@mcopy -D o -i $(diskimage) $(bootloader) ::BOOT.BIN
	@mcopy -D o -i $(diskimage) $(kernel_testing) ::KERNEL.BIN

$(diskimage):
	@mkdir -p $(shell dirname $@)
	@mkfs.msdos -C $(diskimage) 1440

$(diskimage_testing):
	@mkdir -p $(shell dirname $@)
	@mkfs.msdos -C $(diskimage_testing) 1440

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

$(kernel): $(libkernel)
	@ld -o $(kernel) --gc-sections -m elf_i386 -T $(kernel_linker) $(libkernel)

$(libkernel): $(kernel_deps)
	@cd kernel && \
	cargo xbuild --lib --target i386-kernel.json --release
	@cp kernel/target/i386-kernel/release/libkernel.a $(libkernel)

$(kernel_testing): $(libkernel_testing)
	@ld -o $(kernel_testing) --gc-sections -m elf_i386 -T $(kernel_linker) $(libkernel_testing)

$(libkernel_testing): $(kernel_deps)
	@cd kernel && \
	cargo xbuild --lib --target i386-kernel.json --release --features "testing"
	@cp kernel/target/i386-kernel/release/libkernel.a $(libkernel_testing)

$(initfs): $(testexec) $(testcom) $(testdriver) $(testecho) $(dosio) $(elftest) $(command)
	@ls initfs/ | cpio -D initfs -H bin -o > $(initfs)

# System programs:
$(testexec): testexec/test.s
	@as --32 -march=i386 -o build/testexec.o testexec/test.s
	@ld -o $(testexec) --oformat binary -e start -m elf_i386 -Ttext 0 build/testexec.o

$(testcom): testexec/com.s
	@as --32 -march=i386 -o build/testcom.o testexec/com.s
	@ld -o $(testcom) --oformat binary -e start -m elf_i386 -Ttext=0x100 build/testcom.o

$(testdriver): testexec/driver.s
	@as --32 -march=i386 -o build/testdriver.o testexec/driver.s
	@ld -o $(testdriver) --oformat binary -e start -m elf_i386 -Ttext 0 build/testdriver.o

$(testecho): testexec/echo.s
	@as --32 -march=i386 -o build/testecho.o testexec/echo.s
	@ld -o $(testecho) --oformat binary -e start -m elf_i386 -Ttext 0 build/testecho.o

$(dosio): testexec/dosio.s
	@as --32 -march=i386 -o build/dosio.o testexec/dosio.s
	@ld -o $(dosio) --oformat binary -e start -m elf_i386 -Ttext=0x100 build/dosio.o

$(elftest): testexec/elftest.c
	@gcc -shared -nostdlib -nodefaultlibs -fno-exceptions -nostartfiles -fPIE -march=i386 -m32 -Wl,-static -Wl,-Bsymbolic -o $(elftest) testexec/elftest.c

$(command): testexec/command.c
	@gcc -shared -nostdlib -nodefaultlibs -fno-exceptions -nostartfiles -fPIE -march=i386 -m32 -Wl,-static -Wl,-Bsymbolic -o $(command) testexec/command.c
