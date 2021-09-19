use crate::dos::state::VMState;
use crate::fs::DRIVES;
use crate::loaders;
use crate::task::switching::get_current_process;
use super::regs::EnvironmentRegisters;
use super::vm::Subsystem;
use syscall::result::SystemError;

/// Load an executable file from disk, map it into memory, and begin execution
pub fn exec(path_str: &str, interp_mode: loaders::InterpretationMode) -> Result<(), SystemError> {
  let (drive_id, local_handle, env) = loaders::load_executable(path_str, interp_mode).map_err(|e| e.to_system_error())?;
  let to_close = {
    let process_lock = get_current_process();
    let mut process = process_lock.write();
    let old_exec = process.prepare_exec_mapping(env.segments);
    // Remove the old exec and mmap mappings

    // If running a DOS program, the VM needs to be initialized
    if env.require_vm {
      // Set up the memory for the environment
      process.subsystem = Subsystem::DOS(VMState::new());
    }

    process.set_exec_file(drive_id, local_handle)
  };
  // Close the old executable
  match to_close {
    Some((close_drive, close_handle)) => {
      let (_, instance) = DRIVES.get_drive_instance(&close_drive).ok_or(SystemError::NoSuchFileSystem)?;
      instance.close(close_handle).map_err(|_| SystemError::IOError)?;
    },
    None => (),
  }
  // Set up the environment to run the new program
  // Merge the previous register state with the requested state

  // Return the kernel stack pointer to the top of the stack. The next time the
  // process makes a syscall, the stack should be fresh
  get_current_process().write().reset_stack_pointer();

  let mut flags = 0x200;
  if env.require_vm {
    flags |= 0x20000;
  }

  // Prepare the return to userspace
  let regs = EnvironmentRegisters {
    flags,
    edi: 0,
    esi: 0,
    ebp: 0,
    esp: 0xbffffffc,
    ebx: 0,
    edx: 0,
    ecx: 0,
    eax: 0,

    ss: env.registers.ss.unwrap_or(0),
    cs: env.registers.cs.unwrap_or(0),
    eip: env.registers.eip.unwrap_or(0),
  };
  // IRETD requires that we push
  //   Stack Segment
  //   Stack Pointer
  //   Eflags
  //   Code Segment
  //   Instruction Pointer
  // 
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
