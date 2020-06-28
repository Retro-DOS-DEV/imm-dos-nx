use alloc::sync::Arc;
use crate::gdt;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod id;
pub mod map;
pub mod process_state;

static mut PROCESS_MAP: Option<RwLock<map::ProcessMap>> = None;

pub fn init() {
  unsafe {
    PROCESS_MAP = Some(RwLock::new(map::ProcessMap::new()));
  }
}

pub fn all_processes() -> RwLockReadGuard<'static, map::ProcessMap> {
  unsafe {
    match &PROCESS_MAP {
      Some(lock) => lock.read(),
      None => {
        panic!("Process Map not initialized");
      }
    }
  }
}

pub fn all_processes_mut() -> RwLockWriteGuard<'static, map::ProcessMap> {
  unsafe {
    match &PROCESS_MAP {
      Some(lock) => lock.write(),
      None => {
        panic!("Process Map not initialized");
      }
    }
  }
}

pub fn current_process() -> Option<Arc<process_state::ProcessState>> {
  let map = all_processes();
  match map.get_current_process() {
    Some(p) => Some(p.clone()),
    None => None,
  }
}

pub fn make_current(pid: id::ProcessID) {
  let mut map = all_processes_mut();
  map.make_current(pid);
}

pub fn switch_to(pid: id::ProcessID) {
  let (pagedir, esp) = {
    let mut map = all_processes_mut();
    map.make_current(pid);
    let current = map.get_current_process().unwrap();
    let next = map.get_process(pid).unwrap();
    unsafe {
      gdt::set_tss_stack_pointer(next.get_kernel_stack_pointer() as u32);
    }
    let pagedir = next.get_page_directory().get_address().as_usize();
    let esp = next.get_kernel_stack_pointer();
    (pagedir, esp)
  };
  unsafe {
    switch_inner(pagedir, esp);
  }
}

#[naked]
#[inline(never)]
unsafe fn switch_inner(pagedir: usize, esp: usize) {
  llvm_asm!("
    mov cr3, $0
    mov esp, $1
    iretd" : :
    "r"(pagedir), "r"(esp) : :
    "intel", "volatile"
  );
}