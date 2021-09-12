# IMM-DOS NX
The next iteration of an experimental DOS-compatible OS.

The original [IMM-DOS](https://medium.com/@andrewimm/writing-a-dos-clone-in-2019-70eac97ec3e1) was an experiment in building a DOS clone: a real-mode OS for x86 processors that implemented the DOS API and imitated many of its behaviors. **NX** takes it a significant step further, implementing a protected-mode OS that runs real-mode DOS programs in Virtual 8086 environments.

IMM-DOS NX is a multitasking 32-bit OS. Generally speaking, it is designed to support technologies that would have been present in a 90s-era PC. While it is a serious attempt to learn the ins and outs of writing an actually-usable, modern-ish operating system, it shouldn't be taken seriously. It's not meant to replace anything -- this is just an example of building something for the sake of learning. It's a fun what-if exercise exploring how DOS might have evolved into the 90s, if not for the advent of Windows.

All of the core systems and drivers are implemented natively using a custom set of system APIs. When a DOS executable is run, the OS creates a process running in Virtual 8086 mode, gives it access to a DOS-compatible API through `int 21h`, and emulates system hardware like VGA memory. DOS system calls are implemented through the kernel's functionality. For example, access to filesystem from DOS is routed through native filesystem and device drivers. All of this allows DOS applications to run alongside native 32-bit programs and drivers.

## Goals

Beyond functioning as a bootable OS for x86-based systems, there are a few defining goals for what the project seeks to accomplish:

 - 32-bit kernel and drivers
 - Ability to run DOS EXE and COM files in a 8086 VM
 - User-mode device and filesystem drivers
 - Multitasking with terminal multiplexing
 - Support for multiple filesystems
 - System internals available through virtual drives (`DEV:`, `PROC:`, etc)
 - SoundBlaster 16 driver
 - Expanded Memory Manager (EMM)

There are also a few reach goals:

 - VGA Framebuffer drivers
 - Multi-user support
 - Networking (using the FTP Software "Packet Driver Specification")
 - System-wide support for UTF-8
 - DOS Protected Mode Interface (DPMI)
 - Extend the EMM to support Microsoft's Global EMM Import spec

It is explicitly **not** intended to be a POSIX system. At some point during development, it became apparent that the OS was becoming just another UNIX-like. In order to make it more unique and give it a clear purpose, these guidelines were formalized.

## Building and Running

**Dependencies:**
 - Rust 1.55 Nightly or later
 - GNU assembler (`as`), linker (`ld`), and `make`
 - `cpio` archiving tool, for creating the InitFS
 - mtools, for creating a FAT-formatted disk image

At the moment, the OS is only designed to run from a floppy disk image. To build the disk, run `make` from the root directory. This creates a disk image at `build/bootdisk.img` which can be run in a VM like `QEMU`.

## Design

The kernel is a protected mode, 32-bit program that runs DOS applications in a Virtual 8086 (VM86) environment. At its core, it is just a VM86 monitor with hooks to handle interrupts and privileged actions.

The core of the kernel runs at ring 0, with most services and drivers running at ring 3. Memory is virtualized to prevent applications from accidentally clobbering each other. Process separation is less about system security, and more about providing stability (no Guru Meditations!).

### System Components

### Drivers

The NX kernel is multitasking and runs drivers as daemon processes. Some of these are compiled within the kernel and run at ring 0. Others are designed to run in user mode (ring 3), and can be executed when the system launches.

Drivers may provide device or filesystem access. Device drivers provide read/write access to hardware like disks and peripherals, while filesystem drivers implement file IO semantics, often on top of a block device. Installable block and character device drivers make it easier to extend support for different classes of hardware. They also reduce the size and complexity of the kernel.

User mode drivers communicate with the kernel through a process called the Inter-Process IO Arbiter (IPIOA). This is similar to FUSE on Linux, and handles message-passing between IO syscalls and the drivers that respond to them.

Drivers can also install DOS hooks to provide backwards compatibility with legacy programs. Often, these are installed at `int 2Fh`, the multiplexing interrupt. DOS programs that call these interrupts will have their register state passed to the driver so that it can process the request.

### Filesystem

Filesystems aren't just for disks anymore! The NX kernel uses virtual filesystems to expose access to devices and OS internals as well. While this may seem like a very Unix-y shift, it's important to remember that MS-DOS originally used special filenames like `COM1`, `CON`, `AUX`, or `PRN`. Virtual filesystems lead to better organization of these special files, and make debugging the OS much easier.

IMM-DOS NX preserves the `C:\` style of representing absolute file paths, but allows drive names to be more than one character long. Device files are located under the `DEV:` filesystem, and files used for initialization are in `INIT:`. Physical or emulated disk drives are encouraged to use single-letter names, though, since only those disks will be available to DOS programs.

As mentioned above, filesystems can be compiled into the kernel, or loaded as userspace daemons. In order to bootstrap itself, the kernel is launched alongside a simple in-RAM filesystem (init-fs) containing common drivers and utilities needed to boot from disk.

### DOS Virtualization

When the system loads a DOS executable, it creates a new Virtual 8086 VM using behavior built into x86 protected mode. This container uses virtual memory to create the appearance of an entire PC memory area. The kernel sets up in-memory data structures like the PSP, as well as some DOS internal tables to support programs that manually modify these. Other DOS internals are handled within the kernel, to ensure more stable operation of DOS applications.

When DOS syscalls are initiated using interrupts like `int 21h`, the kernel intercepts the call and performs the requested action. For example, a filesystem operation would be proxied through the kernel's filesystem driver, and the result would be sent back to DOS. For other actions that modify memory, like creating a new PSP for a child process, the kernel directly modifies the DOS virtual memory area before returning control.

### Terminal Multiplexing

DOS traditionally runs a single task at a time, taking over the entire screen. Even with a multitasking kernel, the user interface still appears single-tasked. To allow users to use multiple DOS applications at once, the kernel contains a terminal multiplexer that virtualizes video memory. Each terminal instance runs its own version of the command shell, and the child processes it launches modify their own copy of video memory.

The currently visible terminal can be changed with a keyboard hook. The terminal multiplexer is the only process with direct access to VGA memory; when other processes write to video memory, they are actually modifying a separate buffer that the multiplexer can sync with video memory. Keyboard input is only sent to the visible terminal. This is similar to how the Linux console works.

### Memory

The system is implemented as a higher-half kernel -- it exists above `0xc0000000` in virtual memory. This allows usermode programs to exist in the lower `0xbfffffff` bytes without needing to switch page tables each time a syscall is entered. This also makes it easier for the kernel to reference memory in the current process when running syscalls.

Each process has its own mapping for the lower 3GiB. All processes share the same mappings for kernel code and heap, so any process can execute a syscall. Each process also has its own kernel-mode stack to allow for preemptive multitasking.

When native 32-bit programs are run, they are loaded into lower memory. They have a pre-mapped stack at the top of their memory region, and a heap above the top of executable memory. The stack grows downwards, while the heap area grows upwards -- it can be extended with POSIX-style `brk`/`sbrk` syscalls. Arbitrary memory can also be allocated with `mmap` syscalls.

When a DOS program is run, a process is created with a simple linear memory area resembling that of a 16-bit PC. Room is carved out for x86 functionality like the interrupt table, as well as some DOS internals like file handle tables, and the PSP of the executable is placed above that. The executable segments are copied above that.

If a DOS program attempts to directly access BIOS ROM or VGA memory, simulated versions of those are mapped to the appropriate areas, preserving the illusion that the program is running on a 16-bit PC.

### Command Shell

Each new terminal launches an instance of the Command Shell, with the proper files attached to STDIN, STDOUT, and STDERR. The shell presents a DOS-style prompt, and understands general DOS semantics. It contains all of the built-ins you would expect from a DOS-compatible system. 

## Boot Sequence

At the moment, the OS is designed to be booted off of a 1.44M floppy disk. When block drivers are better established, it should be possible to install it on a hard drive and run a persistent system from there.

### Bootloader

The OS is built as a floppy disk image. The master boot record contains a simple first-stage bootloader that reads the FAT root directory, finds the second-stage bootloader, and copies it into memory before executing it.

The second-stage bootloader finds the Kernel and InitFS files on disk, and copies them above the 1MiB mark. It copies them from disk using Real mode interrupts, and then uses "Unreal" mode to copy bytes above `0x100000`. While in Real mode, it also determines how much memory is present in the system. With the files in place, it enters Protected mode, reads the entry point from the kernel's ELF header, and jumps into Rust code.

### Kernel

The kernel starts execution in 32-bit Protected mode, with the stack located somewhere below 1MiB. It immediately clears the `.bss` section, sets up paging and tables for memory management, and sets up a heap allocator. This allows the kernel code to use heap-allocated Rust objects like `Box` and `Vec`. The kernel leverages these to start initializing filesystems and core device drivers. It also sets up the structures to track processes and switch between tasks. This initial process then forks and jumps to user mode, and becomes the idle process that halts the CPU when no other processes are running.

### Init Process

Because drivers are meant to be loaded, rather than compiled into the kernel, the kernel itself ships with minimal device and filesystem drivers. It instead loads all necessary drivers and utilities from an in-memory archive called InitFS that gets loaded alongside the kernel. The InitFS is simply a CPIO archive, similar to what was used for Linux's `initramfs`. This makes it easy to create and traverse.

The first user-mode process executes the `init` binary, which is used to spawn all other processes. It initializes device and fileystem drivers needed to actually use the system, and creates the initial TTY terminals.

Much of this `init` stage has yet to be built out. This section will be updated when there are more details to share.

## Notes

This code is for demonstration purposes, and is licensed under the terms found in the LICENSE file in the root of this repository.

I am providing code in the repository to you under an open source license. Because this is my personal repository, the license you receive to my code is from me and not my employer (Facebook).
