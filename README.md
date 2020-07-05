# IMM-DOS NX
The next iteration of an experimental DOS-compatible OS.

The original [IMM-DOS](https://medium.com/@andrewimm/writing-a-dos-clone-in-2019-70eac97ec3e1) was an experiment in building a DOS clone: a real-mode OS for x86 processors that implemented the DOS API and imitated many of its behaviors. **NX** takes it a significant step further, implementing a protected-mode OS that runs real-mode DOS programs in Virtual 8086 environments.

IMM-DOS NX is a multitasking 32-bit OS. Generally speaking, it is designed to support technologies that would have been present in a 90s-era PC. While it is a serious attempt to learn the ins and outs of writing an actually-usable, modern-ish operating system, it shouldn't be taken seriously. It's not meant to replace anything -- this is just an example of building something for the sake of learning. It's a fun what-if exercise exploring how DOS might have evolved into the 90s, if not for the advent of Windows.

All of the system applications and drivers are implemented natively using custom system APIs. When a DOS executable is run, the OS creates a process running in Virtual 8086 mode, gives it access to a DOS-compatible API through `int 21h`, and emulates system hardware like VGA memory. DOS system calls are implemented through the kernel's functionality. For example, access to filesystem from DOS is routed through native filesystem and device drivers. All of this allows DOS applications to run alongside native 32-bit programs.

## Building and Running

**Dependencies:**
 - Rust 1.45 Nightly or later
 - GNU assembler (`as`), linker (`ld`), and `make`
 - `cpio` archiving tool, for creating the InitFS
 - mtools, for creating a FAT-formatted disk image

At the moment, the OS is only designed to run from a floppy disk image. To build the disk, run `make` from the root directory.

## Design

In many ways, the architecture of the OS resembles that of a modern multitasking Unix descendant. In other ways, it runs forward with some DOS-isms to make the user experience feel like a natural evolution of DOS.

### Filesystem

Filesystems aren't just for disks anymore! The NX kernel uses virtual filesystems to expose access to devices and OS internals as well. While this may seem like a very Unix-y shift, it's important to remember that MS-DOS originally used special filenames like `COM1`, `CON`, or `PRN`. Virtual filesystems lead to better organization of these special files, and make debugging the OS much easier.

IMM-DOS NX preserves the `C:\` style of representing absolute file paths, but allows drive names to be more than one character long. Device files are located under the `DEV:` filesystem, and files used for initialization are in `INIT:`. Physical or emulated disk drives are encouraged to use single-letter names, though, since only those disks will be available to DOS programs.

Filesystems are not compiled into the kernel. Instead, they are implemented as userspace daemons that communicate with the kernel. In order to bootstrap itself, the kernel is launched alongside a simple in-RAM filesystem containing common drivers and utilities needed to boot from disk. This is covered in more detail in the Boot Sequence section below.

### Memory

The core of the code is implemented as a higher-half kernel -- it exists above `0xc0000000` in virtual memory. This allows user programs to exist in the lower `0xbfffffff` bytes without needing to switch page tables each time a syscall is entered. This also makes it easier for the kernel to reference memory in the current process when running syscalls.

**Memory Map:**

| Address | Role |
| ------- | ---- |
| **0xffc00000 - 0xffffffff** | Identity mapped page directory, for easily editing paging |
| **0xffbff000 - 0xffbfffff** | Temporary space for editing unmapped pages |
| **0xffbfe000 - 0xffbfefff** | Top page of kernel stack. The stack extends lower as needed |
| **0xc0000000 and up** | Kernel code and heap, and in-memory copy of InitFS |
| **0xbfffffff and down** | Userspace stack, extending downwards |
| **???** | Userspace `brk` heap, immediately following data sections, extending upwards |
| **0x00000000 and up** | Userspace code and data |

Every process has its own mapping for the lower 3GiB, and its own kernel stack. All processes share the same mappings for kernel code and heap, so any process can execute a syscall.

### Subsystems

The NX kernel is designed to be able to run Real-mode DOS programs and native 32-bit applications side by side. In order to accomplish this, processes can be associated with different *subsystems*. At the moment only two subsystems have been scoped out: one for "native" apps using the NX syscalls in 32-bit protected mode, and one for 16-bit DOS programs running in an 8086 VM. Each subsystem exposes its own set of syscalls, though all syscalls have access to the same kernel internals. The only way to change subsystems is through the `exec()` syscall, which also loads new code for the process, so it is not easily possible for already-running code to change its subsystem and continue running.

To run DOS programs using Virtual 8086 mode, the kernel needs to do some extra bookkeeping for each process. These values are stored in the subsystem metadata attached to the process's state. DOS programs trigger syscalls through the established set of DOS interrupts, chiefly among them `int 21h`. Any interrupt coming from a DOS program triggers a General Protection Fault, which is intercepted by the kernel and handled accordingly.

## Boot Sequence

At the moment, the OS is designed to be booted off of a 1.44M floppy disk. When block drivers are better established, it should be possible to install it on a hard drive and run a system continously from there.

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
