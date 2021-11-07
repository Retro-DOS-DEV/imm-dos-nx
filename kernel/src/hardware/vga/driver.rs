use crate::memory::address::{SegmentedAddress, VirtualAddress};
use crate::memory::physical::{self, frame::Frame};
use crate::memory::virt::page_directory::{CurrentPageDirectory, PageDirectory, PermissionFlags};
use crate::task::regs::EnvironmentRegisters;

/// The only reliable way to switch video modes is to use the code copied to
/// BIOS for the installed video card. This is possible by spinning up a
/// virtual 8086 VM that has access to BIOS code.
/// The VGA Driver Process creates this mapping, and listens for requests from
/// the kernel to change video modes. When a request comes in, it enters 8086
/// mode with a set of registers, simulates the INT 10h call, and changes the
/// video mode. When the request completes, it tells the kernel to unblock the
/// caller so that it can resume execution in the new video mode.
pub extern "C" fn vga_driver_process() {
  let pagedir = CurrentPageDirectory::get();
  // Allocate the lowest frame of physical memory to its same location
  pagedir.map(Frame::new(0), VirtualAddress::new(0), PermissionFlags::new(PermissionFlags::USER_ACCESS));
  // Allocate the BIOS code area (0xA0000 - 0xFFFFF)
  let mut frame = 0xA0000;
  while frame < 0x100000 {
    pagedir.map(Frame::new(frame), VirtualAddress::new(frame), PermissionFlags::new(PermissionFlags::USER_ACCESS));
    frame += 0x1000;
  }

  crate::kprintln!("Video Driver Ready");

  let stack_frame = physical::allocate_frame().unwrap();
  pagedir.map(stack_frame, VirtualAddress::new(0x7f000), PermissionFlags::new(PermissionFlags::USER_ACCESS));

  let on_return_addr = return_from_interrupt as *const extern "C" fn() -> () as usize;
  crate::task::get_current_process().write().on_exit_vm = Some(on_return_addr);

  // Set up fake code for our demo
  unsafe {
    // `IRET`
    *(0x500 as *mut u8) = 0xcf;
  }

  wait_for_message();
}

extern "C" fn wait_for_message() {
  loop {
    change_mode();
  }
}

extern "C" fn change_mode() {
  let int_10_address: &SegmentedAddress = unsafe {
    &*(0x40 as *const SegmentedAddress)
  };
  let mode = 0x13;
  // jump to INT 10h
  let mut regs = EnvironmentRegisters {
    eax: mode,
    ecx: 0,
    edx: 0,
    ebx: 0,
    ebp: 0,
    esi: 0,
    edi: 0,

    //eip: int_10_address.offset as u32,
    //cs: int_10_address.segment as u32,
    eip: 0x500,
    cs: 0,
    flags: 0x20200,
    esp: 0xffe,
    ss: 0x7f00,

    es: 0x7f00,
    ds: 0x7f00,
    fs: 0x7f00,
    gs: 0x7f00,
  };
  // set up the stack
  unsafe {
    // push flags
    *(0x7fffe as *mut u16) = 0;
    regs.esp -= 2;
    // push cs
    *(0x7fffc as *mut u16) = 0x00;
    regs.esp -= 2;
    // push ip
    *(0x7fffa as *mut u16) = 0;
    regs.esp -= 2;
  }

  // copied from task::exec, can these be combined?
  unsafe {
    asm!(
      "cld
      mov ecx, ({regs_size} / 4)
      mov edi, esp
      sub edi, 4 + {regs_size}
      mov esi, eax
      rep
      movsd
      sub esp, 4 + {regs_size}
      pop eax
      pop ecx
      pop edx
      pop ebx
      pop ebp
      pop esi
      pop edi
      iretd",
      regs_size = const core::mem::size_of::<EnvironmentRegisters>(),
      // can't directly use esi as an input because LLVM
      in("eax") (&regs as *const EnvironmentRegisters as usize),
      options(noreturn),
    );
  }
}

extern "C" fn return_from_interrupt() {
  crate::kprintln!("Returned from VM86");
  loop {
    crate::task::yield_coop();
  }
}
