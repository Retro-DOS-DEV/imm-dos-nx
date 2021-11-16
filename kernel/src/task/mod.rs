#[cfg(not(test))]
pub mod exec;
pub mod files;
pub mod id;
pub mod io;
pub mod ipc;
pub mod memory;
#[cfg(not(test))]
pub mod paging;
pub mod process;
pub mod regs;
pub mod signal;
pub mod stack;
pub mod state;
#[cfg(not(test))]
pub mod switching;
pub mod vm;

#[cfg(not(test))]
pub use switching::yield_coop;
#[cfg(test)]
pub fn yield_coop() {}

#[cfg(not(test))]
pub fn sleep(duration: usize) {
  let current_lock = switching::get_current_process();
  current_lock.write().sleep(duration);
  yield_coop();
}
#[cfg(test)]
pub fn sleep(_duration: usize) {}

#[cfg(not(test))]
pub fn fork() -> id::ProcessID {
  let current_ticks = crate::time::system::get_system_ticks();
  switching::fork(current_ticks, true)
}

#[cfg(not(test))]
pub fn wait(child_id: Option<id::ProcessID>) -> u32 {
  let current = switching::get_current_process();
  current.write().wait(child_id);
  yield_coop();
  let code = current.write().resume_from_wait();
  code
}

#[cfg(not(test))]
pub use switching::get_current_process;
#[cfg(test)]
pub fn get_current_process() -> alloc::sync::Arc<spin::RwLock<process::Process>> {
  panic!("No current process in test");
}

#[cfg(not(test))]
pub use switching::get_process;
#[cfg(test)]
pub fn get_process(_id: &id::ProcessID) -> Option<alloc::sync::Arc<spin::RwLock<process::Process>>> {
  panic!("Not available in test");
}

#[cfg(not(test))]
pub use switching::get_current_id;
#[cfg(test)]
pub fn get_current_id() -> id::ProcessID {
  id::ProcessID::new(0)
}

#[cfg(not(test))]
pub use exec::terminate;
#[cfg(test)]
pub fn terminate(_exit_code: u32) {}

#[cfg(not(test))]
pub fn ipc_read(timeout: Option<usize>) -> (Option<ipc::IPCPacket>, bool) {
  let current_ticks = crate::time::system::get_system_ticks();
  let (first, has_more) = {
    let current_process_lock = switching::get_current_process();
    let mut current_process = current_process_lock.write();
    current_process.ipc_read(current_ticks, timeout)
  };
  if first.is_some() {
    return (first, has_more);
  }
  yield_coop();
  switching::get_current_process().write().ipc_read_unblocking(current_ticks)
}

#[cfg(not(test))]
pub fn ipc_send(to: id::ProcessID, message: ipc::IPCMessage, expiration: u32) {
  let current_id = switching::get_current_id();
  let current_ticks = crate::time::system::get_system_ticks();
  let recipient = switching::get_process(&to);
  if let Some(rec_lock) = recipient {
    rec_lock.write().ipc_receive(current_ticks, current_id, message, expiration);
  }
}
