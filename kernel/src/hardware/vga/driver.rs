use core::sync::atomic::{AtomicU8, Ordering};
use crate::memory::address::{PhysicalAddress, SegmentedAddress, VirtualAddress};
use crate::memory::physical::{self, frame::Frame};
use crate::memory::virt::page_directory::{CurrentPageDirectory, PermissionFlags};
use crate::task::id::ProcessID;
use crate::task::ipc::{IPCMessage, IPCPacket};
use crate::task::regs::EnvironmentRegisters;
use spin::RwLock;

/// Stores the ProcessID of the VGA Driver once it is initialized
pub static VGA_DRIVER_PID: RwLock<Option<ProcessID>> = RwLock::new(None);
/// Stores the ID of the process that most recently made a request to the
/// driver. On return from VM86 mode, this process is resumed.
static CURRENT_REQUEST_PID: RwLock<Option<ProcessID>>  = RwLock::new(None);
/// Stores the last confirmed setting of the video mode
static CURRENT_VIDEO_MODE: AtomicU8 = AtomicU8::new(0x03);

pub const MSG_MODE_SWITCH: u32 = 1;

/// The only reliable way to switch video modes is to use the code copied to
/// BIOS for the installed video card. This is possible by spinning up a
/// virtual 8086 VM that has access to BIOS code.
/// The VGA Driver Process creates this mapping, and listens for requests from
/// the kernel to change video modes. When a request comes in, it enters 8086
/// mode with a set of registers, simulates the INT 10h call, and changes the
/// video mode. When the request completes, it tells the kernel to unblock the
/// caller so that it can resume execution in the new video mode.
pub extern "C" fn vga_driver_process() {
  let current_id = crate::task::switching::get_current_id();
  *VGA_DRIVER_PID.write() = Some(current_id);

  let pagedir = CurrentPageDirectory::get();
  // Allocate the lowest frame of physical memory to its same location
  pagedir.map_explicit(PhysicalAddress::new(0), VirtualAddress::new(0), PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS));
  // Allocate the BIOS code area (0xA0000 - 0xFFFFF)
  let mut frame = 0xA0000;
  while frame < 0x100000 {
    pagedir.map_explicit(PhysicalAddress::new(frame), VirtualAddress::new(frame), PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS));
    frame += 0x1000;
  }

  crate::kprintln!("Video Driver Ready");

  let stack_frame = physical::allocate_frame().unwrap();
  pagedir.map(stack_frame, VirtualAddress::new(0x7f000), PermissionFlags::new(PermissionFlags::USER_ACCESS | PermissionFlags::WRITE_ACCESS));

  let on_return_addr = return_from_interrupt as *const extern "C" fn() -> () as usize;
  crate::task::get_current_process().write().on_exit_vm = Some(on_return_addr);

  wait_for_message();
}

/// Public API to send a request to the driver via IPC. The current process will
/// be blocked on hardware IO until the driver returns from the request.
pub fn send_request(message: IPCMessage, timeout: Option<usize>) {
  let driver_id = *VGA_DRIVER_PID.read();
  match driver_id {
    Some(id) => {
      crate::task::ipc_send(id, message, 0xffffffff);
    },
    None => return,
  }

  crate::task::get_current_process().write().hardware_block(timeout);
  crate::task::yield_coop();
}

/// Request a VGA graphics mode change
pub fn request_mode_change(mode: u8) {
  let message = IPCMessage(MSG_MODE_SWITCH, mode as u32, 0, 0);
  send_request(message, None);
}

/// Request a VGA graphics mode change
pub fn request_mode_change_with_timeout(mode: u8, timeout: usize) {
  let message = IPCMessage(MSG_MODE_SWITCH, mode as u32, 0, 0);
  send_request(message, Some(timeout));
}

/// Fetch the current known video mode
pub fn get_video_mode() -> u8 {
  CURRENT_VIDEO_MODE.load(Ordering::SeqCst)
}

/// Internal logic for the graphics driver. It blocks on IPC requests until one
/// is received, and parses that message to determine how to modify the VGA
/// hardware.
extern "C" fn wait_for_message() {
  loop {
    let (ipc_packet, _) = crate::task::ipc_read(None);
    match ipc_packet {
      Some(IPCPacket { from, message }) =>
        match message {
          IPCMessage(MSG_MODE_SWITCH, mode, _, _) => {
            *CURRENT_REQUEST_PID.write() = Some(from);
            change_mode(mode);
          },
          _ => {
            // unknown packet, just wake the caller
            crate::task::switching::get_process(&from)
              .and_then(|lock| Some(lock.write().hardware_resume()));
          },
        },
      _ => (),
    }
  }
}

extern "C" fn change_mode(mode: u32) {
  let int_10_address: &SegmentedAddress = unsafe {
    &*(0x40 as *const SegmentedAddress)
  };
  // jump to INT 10h
  let mut regs = EnvironmentRegisters {
    eax: mode,
    ecx: 0,
    edx: 0,
    ebx: 0,
    ebp: 0,
    esi: 0,
    edi: 0,

    eip: int_10_address.offset as u32,
    cs: int_10_address.segment as u32,
    flags: 0x20200,
    esp: 0xfffe,
    ss: 0x7000,

    es: 0x7000,
    ds: 0x7000,
    fs: 0x7000,
    gs: 0x7000,
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
  let current_video_mode = unsafe {
    *(0x449 as *const u8)
  };
  CURRENT_VIDEO_MODE.store(current_video_mode, Ordering::SeqCst);

  let request_id = CURRENT_REQUEST_PID.write().take();
  request_id
    .and_then(|id| crate::task::switching::get_process(&id))
    .and_then(|proc| Some(proc.write().hardware_resume()));

  if current_video_mode == 0x13 {
    unsafe {
      let base = 0xa0000 as *mut u8;
      for row in 0..8 {
        for i in 0..256 {
          core::ptr::write_volatile(base.offset(row * 320 + i), i as u8);
        }
      }
    }
  }

  wait_for_message();
}
