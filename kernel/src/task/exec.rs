use crate::dos::state::VMState;
use crate::fs::DRIVES;
use crate::loaders;
use crate::memory::address::VirtualAddress;
use crate::task::switching::{get_current_process, yield_coop};
use super::regs::EnvironmentRegisters;
use super::vm::Subsystem;
use syscall::result::SystemError;

/// Load an executable file from disk, map it into memory, and begin execution
pub fn exec(path_str: &str, interp_mode: loaders::InterpretationMode) -> Result<(), SystemError> {
  let (drive_id, local_handle, env) = loaders::load_executable(path_str, interp_mode).map_err(|e| e.to_system_error())?;
  // TODO: If anything fails within or after this block, we need a way to
  // "rewind" the changes here.
  let to_close = {
    let process_lock = get_current_process();
    let mut process = process_lock.write();
    let old_exec = process.prepare_exec_mapping(env.segments);
    // Remove the old exec and mmap mappings:
    let heap_range = process.memory.get_heap_page_range();
    super::paging::unmap_task(old_exec, heap_range);

    // Map a new stack frame, and push arguments onto it

    // If running a DOS program, the VM needs to be initialized
    if env.require_vm {
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
  if env.require_vm {
    // Initialize DOS memory
    let segment = env.registers.cs.unwrap_or(0) as u16;
    let psp = unsafe { crate::dos::execution::PSP::at_segment(segment) };
    // Writing to this PSP will trigger a page fault and fill the first page of
    // the program.
    psp.reset();
  }
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

    gs: 0,
    fs: 0,
    es: env.registers.es.unwrap_or(0x20 | 3),
    ds: env.registers.ds.unwrap_or(0x20 | 3),

    ss: env.registers.ss.unwrap_or(0x20 | 3),
    cs: env.registers.cs.unwrap_or(0x18 | 3),
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

pub fn terminate(exit_code: u32) {
  let current_process = get_current_process();
  let (cur_id, parent_id) = {
    let mut cur = current_process.write();
    cur.terminate();
    (*cur.get_id(), *cur.get_parent_id())
  };
  {
    let parent_lock = super::switching::get_process(&parent_id);
    if let Some(parent) = parent_lock {
      parent.write().child_returned(cur_id, exit_code);
    }
  }
  yield_coop();
}

pub fn set_heap_top(addr: VirtualAddress) -> Result<VirtualAddress, ()> {
  let current_process_lock = get_current_process();
  let mut cur = current_process_lock.write();
  let heap_start = cur.memory.get_heap_start();
  if addr < heap_start {
    return Err(());
  }
  let size = addr - heap_start;
  cur.memory.set_heap_size(size);
  Ok(cur.memory.get_heap_start() + cur.memory.get_heap_size())
}

pub fn move_heap_top(delta: isize) -> Result<VirtualAddress, ()> {
  let current_process_lock = get_current_process();
  let mut cur = current_process_lock.write();
  if delta != 0 {
    let current_size = cur.memory.get_heap_size();
    let new_size = current_size as isize + delta;
    if new_size < 0 {
      return Err(());
    } else {
      cur.memory.set_heap_size(new_size as usize);
    }

    // if the heap shrank, do we need to unmap pages?
  }

  Ok(cur.memory.get_heap_start() + cur.memory.get_heap_size())
}
