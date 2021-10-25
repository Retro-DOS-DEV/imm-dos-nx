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
pub fn sleep(duration: usize) {}

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
pub use exec::terminate;
#[cfg(test)]
pub fn terminate(exit_code: u32) {}
