#[cfg(not(test))]
pub mod exec;
pub mod files;
pub mod id;
#[cfg(not(test))]
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