use crate::memory::address::VirtualAddress;
use crate::process;
use crate::task;
use syscall::result::SystemError;

pub fn yield_coop() {
  let id = task::switching::get_current_id().as_u32();
  task::yield_coop();
}

pub fn sleep(ms: u32) {
  task::sleep(ms as usize)
}

pub fn fork() -> u32 {
  let id = task::fork();
  id.as_u32()
}

pub fn exec_path(path_str: &'static str, _arg_str: &'static str, raw_interp_mode: u32) -> Result<(), SystemError> {
  let interp_mode = crate::loaders::InterpretationMode::from_u32(raw_interp_mode);
  task::exec::exec(path_str, interp_mode)
}

pub fn exit(code: u32) {
  task::exec::terminate(code);
}

pub fn get_pid() -> u32 {
  task::switching::get_current_id().as_u32()
}

pub fn raise_signal(sig: u32) {
  let id = process::get_current_pid();
  process::send_signal(id, sig);
}

pub fn send_signal(id: u32, sig: u32) {
  process::send_signal(process::id::ProcessID::new(id), sig);
}

pub fn wait_pid(id: u32) -> (u32, u32) {
  if id == 0 {
    let code = task::wait(None);
    (0, code)
  } else {
    let code = task::wait(Some(task::id::ProcessID::new(id)));
    (id, code)
  }
}

pub fn brk(method: u32, offset: u32) -> Result<u32, ()> {
  match method {
    0 => { // Absolute
      let addr = VirtualAddress::new(offset as usize);
      task::exec::set_heap_top(addr).map(|addr| addr.as_u32())
    },
    1 => { // Relative
      let delta = offset as i32 as isize;
      task::exec::move_heap_top(delta).map(|addr| addr.as_u32())
    },
    _ => {
      Err(())
    },
  }
}

pub fn install_interrupt_handler(irq: u32, address: u32, stack_top: u32) -> Result<(), ()> {
  let cur_id = task::switching::get_current_id();
  crate::kprintln!("INSTALL HANDLER AT {}:{:#010x} to IRQ {}", cur_id.as_u32(), address, irq);
  crate::interrupts::handlers::install_handler(
    irq as usize,
    cur_id,
    VirtualAddress::new(address as usize),
    VirtualAddress::new(stack_top as usize),
  )
}
