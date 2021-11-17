use core::mem::size_of;
use crate::interrupts::stack::StackFrame;
use crate::interrupts::syscall_legacy::dos_api;
use super::registers::{DosApiRegisters, VM86Frame};

/// When a DOS program running in VM86 mode tries to do something privileged, it
/// will trigger a GPF. If the kernel GPF handler determines that VM86 mode is
/// active, it will call this method, allowing the kernel to emulate the
/// appropriate behavior.
pub fn handle_gpf(stack_frame: &StackFrame) -> bool {
  let stack_frame_ptr = stack_frame as *const StackFrame as usize;
  let vm_frame_ptr = (stack_frame_ptr + size_of::<StackFrame>()) as *mut VM86Frame;
  // The registers get pushed by the x86-interrupt wrapper.
  // They should be found beneath the last argument for this method.
  let reg_ptr = (
    stack_frame_ptr - size_of::<u32>() - size_of::<DosApiRegisters>()
  ) as *mut DosApiRegisters;
  unsafe {
    let regs = &mut *reg_ptr;
    let vm_frame = &mut *vm_frame_ptr;
    let op_ptr = ((stack_frame.cs << 4) + stack_frame.eip) as *const u8;
    if *op_ptr == 0x66 { // Op32 Prefix
      panic!("Unsupported privileged instruction");
    } else if *op_ptr == 0x67 { // Addr32 Prefix
      panic!("Unsupported privileged instruction");
    } else if *op_ptr == 0x9c { // PUSHF
      // needs to handle 32-bit variant
      vm_frame.sp = (vm_frame.sp - 2) & 0xffff;
      let sp = (vm_frame.ss << 4) + vm_frame.sp;
      *(sp as *mut u16) = stack_frame.eflags as u16;
      stack_frame.add_eip(1);
      return true;
    } else if *op_ptr == 0x9d { // POPF
      // needs to handle 32-bit variant
      let sp = (vm_frame.ss << 4) + vm_frame.sp;
      let flags = *(sp as *const u16);
      vm_frame.sp = (vm_frame.sp + 2) & 0xffff;
      stack_frame.set_eflags((flags as u32) | 0x20200);
      stack_frame.add_eip(1);
      return true;
    } else if *op_ptr == 0xcd {
      // INT
      let interrupt = *op_ptr.offset(1);
      handle_interrupt(interrupt, regs, vm_frame, stack_frame);
      // Compiler will try to optimize out a write to the StackFrame
      stack_frame.add_eip(2);
      return true;
    } else if *op_ptr == 0xcf { // IRET
      let sp = (vm_frame.ss << 4) + vm_frame.sp;
      let (ip, cs, flags) = (
        *(sp as *const u16),
        *((sp + 2) as *const u16),
        *((sp + 4) as *const u16),
      );
      if cs == 0 && ip == 0 {
        // Can't jump to zero, it's the IVT!
        // Use this case as the hook to request exiting VM86 mode.
        // This is pretty much only used by the modesetting graphics driver...
        let fn_resume = {
          let current_process = crate::task::get_current_process();
          let on_exit_vm = current_process.read().on_exit_vm;
          on_exit_vm
        };
        match fn_resume {
          Some(addr) => {
            asm!(
              "jmp eax",
              in("eax") addr,
              options(noreturn),
            );
          },
          None => (),
        }
      }
      // nothing special, perform the iret like normal
      stack_frame.set_eip(ip as u32);
      stack_frame.set_cs(cs as u32);
      stack_frame.set_eflags((flags as u32) | 0x20200);
      // mark virtual interrupt flag from `flags`
      vm_frame.sp = (vm_frame.sp + 6) & 0xffff;
      return true;
    } else if *op_ptr == 0xfa { // CLI
      // clear virtual interrupt flag
      stack_frame.add_eip(1);
      return true;
    } else if *op_ptr == 0xfb { // STI
      // set virtual interrupt flag
      stack_frame.add_eip(1);
      return true;
    }
  }
  false
}

fn handle_interrupt(
  interrupt: u8,
  regs: &mut DosApiRegisters,
  vm_frame: &mut VM86Frame,
  stack_frame: &StackFrame,
) {
  match interrupt {
    0x00 => { // Divide error
      panic!("Unsupported DOS interrupt 0x00");
    },
    0x01 => { // Single-step debug trap
      panic!("Unsupported DOS interrupt 0x01");
    },
    0x02 => { // Hardware NMI
      panic!("Unsupported DOS interrupt 0x02");
    },
    0x03 => { // Breakpoint
      // should yield execution and send a signal to any tracing processes
      panic!("Break");
    },
    0x04 => { // Overflow Interrupt (INTO)
      panic!("Unsupported DOS interrupt 0x04");
    },
    0x05 => { // Print Screen
      panic!("Unsupported DOS interrupt 0x05");
    },
    0x06 => { // Invalid Opcode
      panic!("Unsupported DOS interrupt 0x06");
    },
    0x07 => { // Coprocessor not available
      panic!("DOS Interupt: Coprocessor Unavailable. How did you even do that?");
    },
    0x08 => { // Timer interrupt
      // By default, Timer should run every 55ms (18.21590 times per second)
      panic!("Unsupported DOS interrupt 0x08");
    },
    0x09 => { // Keyboard service request
      // Fetches key data from the keyboard, and puts it in the BIOS buffer
      // where it can be used by INT 0x16
      panic!("Unsupported DOS interrupt 0x09");
    },
    0x0a => { // IRQ2, PIC cascade
      panic!("Unsupported DOS interrupt 0x0a");
    },
    0x0b => { // COM2 interrupt
      panic!("Unsupported DOS interrupt 0x0b");
    },
    0x0c => { // COM1 interrupt
      panic!("Unsupported DOS interrupt 0x0c");
    },
    0x0d => { // Fixed disk, LPT2
      panic!("Unsupported DOS interrupt 0x0d");
    },
    0x0e => { // Floppy disk
      panic!("Unsupported DOS interrupt 0x0e");
    },
    0x0f => { // LPT1 printer interrupt
      panic!("Unsupported DOS interrupt 0x0f");
    },
    0x10 => { // Video request
      video_interrupt(regs);
    },
    0x11 => { // Equipment detection
      panic!("Unsupported DOS interrupt 0x11");
    },
    0x12 => { // Get memory size
      panic!("Unsupported DOS interrupt 0x12");
    },
    0x13 => { // Disk IO
      disk_io(regs);
    },
    0x14 => { // Serial Comms
      serial_comms(regs);
    },
    0x15 => { // System Services
      panic!("Unsupported DOS interrupt 0x15");
    },
    0x16 => { // Keyboard Services
      keyboard_service(regs);
    },
    0x17 => { // Parallel printer
      panic!("Unsupported DOS interrupt 0x17");
    },
    // ...
    0x1a => { // System clock
      panic!("Unsupported DOS interrupt 0x1a");
    },
    0x1b => { // Custom ctrl-break handler
      panic!("Unsupported DOS interrupt 0x1b");
    },
    0x1c => { // Custom clock tick handler
      panic!("Unsupported DOS interrupt 0x1c");
    },
    // ...
    0x20 => { // DOS terminate
      panic!("DOS legacy terminate not implemented");
    },
    0x21 => { // DOS API
      crate::debug::log_dos_syscall(regs.ah());
      dos_api(regs, vm_frame, stack_frame);
    },
    0x22 => {
      // not an interrupt, but the address to jump to on termination
      panic!("DOS int 0x22 is not an interrupt!");
    },
    0x23 => {
      // similar to 0x22, address to jump to on ctrl-break
      panic!("DOS int 0x23 is not an interrupt!");
    },
    0x24 => { // Critical DOS error
      // infamous Abort / Retry / Fail handler
      panic!("DOS critical error handler not implemented");
    },
    0x25 => { // Absolute disk read
      panic!("DOS absolute disk read not implemented");
    },
    0x26 => { // Absolute disk write
      panic!("DOS absolute disk write not implemented");
    },
    0x27 => { // Terminate & Stay Resident
      panic!("DOS TSR not implemented");
    },
    // ...
    0x2f => { // Multiplexed interrupt
      panic!("DOS multiplex interrupt not implemented");
    },
    // ...
    0x31 => { // DPMI
      panic!("DPMI not implemented");
    },
    // ...
    0x33 => { // Mouse Driver
      panic!("DOS mouse driver not implemented");
    },
    _ => panic!("Unsupported interrupt from VM86 mode: {:X}", interrupt),
  }
}

fn video_interrupt(regs: &mut DosApiRegisters) {
  let method = regs.ah();
  match method {
    0x00 => { // set video mode

    },
    0x01 => { // set cursor shape
    },
    0x02 => { // set cursor position
    },
    0x03 => { // get cursor position
    },
    0x04 => { // read light pen position
    },
    0x05 => { // set active video page
    },
    0x06 => { // scroll area up
    },
    0x07 => { // scroll area down
    },
    0x08 => { // get character / value and attribute at cursor
    },
    0x09 => { // write character / value and attribute to cursor
    },
    0x0a => { // write character at cursor
    },
    0x0b => { // set palette
    },
    0x0c => { // write pixel at coordinate
    },
    0x0d => { // read pixel at coordinate
    },
    0x0e => { // write character in TTY mode
    },
    0x0f => { // get video state
    },
    _ => {
      panic!("Unsupported DOS video interrupt method: {:X}", method);
    }
  }
}

fn disk_io(regs: &mut DosApiRegisters) {
}

fn serial_comms(regs: &mut DosApiRegisters) {
}

fn keyboard_service(regs: &mut DosApiRegisters) {
  let method = regs.ah();
  match method {
    0x00 => { // wait for key
    },
    0x01 => { // get key status
    },
    0x02 => { // get shift status
    },
    0x03 => { // set typematic rate
    },
    0x04 => { // click adjustment
    },
    0x05 => { // write to keyboard buffer
    },
    _ => panic!("Unsupported DOS keyboard interrupt method: {:X}", method),
  }
}
