use alloc::sync::Arc;
use crate::gdt;
use crate::kprintln;
use crate::memory::virt;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod id;
pub mod map;
pub mod memory;
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
  let (pagedir, old_proc_esp, new_proc_esp) = {
    let mut map = all_processes_mut();
    let current = map.get_current_process().unwrap();
    let old_proc_esp = current.get_kernel_stack_container() as *const RwLock<usize>;
    kprintln!("Switch from {:?} to {:?}", current.get_id(), pid);
    kprintln!(" Cur esp was {:x}", current.get_kernel_stack_pointer());
    map.make_current(pid);
    let next = map.get_process(pid).unwrap();
    kprintln!(" Next esp is {:x}", next.get_kernel_stack_pointer());
    unsafe {
      gdt::set_tss_stack_pointer(virt::STACK_START.as_u32() + 0x1000 - 4);
    }
    let pagedir = next.get_page_directory().get_address().as_usize();
    let new_proc_esp = next.get_kernel_stack_container() as *const RwLock<usize>;
    (pagedir, old_proc_esp, new_proc_esp)
  };
  unsafe {
    llvm_asm!("push eax; push ecx; push edx; push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    switch_inner(pagedir, old_proc_esp, new_proc_esp);
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx; pop edx; pop ecx; pop eax" : : : "esp" : "intel", "volatile");
  }
}

pub fn enter_usermode(pid: id::ProcessID) {
  let (pagedir, old_proc_esp, new_proc_esp) = {
    let mut map = all_processes_mut();
    let current = map.get_current_process().unwrap();
    let old_proc_esp = current.get_kernel_stack_container() as *const RwLock<usize>;
    map.make_current(pid);
    let next = map.get_process(pid).unwrap();
    unsafe {
      gdt::set_tss_stack_pointer(virt::STACK_START.as_u32() + 0x1000 - 4);
    }
    let pagedir = next.get_page_directory().get_address().as_usize();
    let new_proc_esp = next.get_kernel_stack_container() as *const RwLock<usize>;
    (pagedir, old_proc_esp, new_proc_esp)
  };
  unsafe {
    llvm_asm!("push eax; push ecx; push edx; push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    enter_inner(pagedir, old_proc_esp, new_proc_esp);
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx; pop edx; pop ecx; pop eax" : : : "esp" : "intel", "volatile");
  }
}

#[naked]
#[inline(never)]
unsafe fn switch_inner(pagedir: usize, old_proc_esp: *const RwLock<usize>, new_proc_esp: *const RwLock<usize>) {
  llvm_asm!("mov cr3, $0" : : "r"(pagedir) : : "intel", "volatile");
  let cur_esp;
  llvm_asm!("mov $0, esp" : "=r"(cur_esp) : : : "intel", "volatile");
  *(*old_proc_esp).write() = cur_esp;
  let next_esp = (*new_proc_esp).read().clone();
  llvm_asm!("mov esp, $0" : : "r"(next_esp) : : "intel", "volatile");
}

#[naked]
#[inline(never)]
unsafe fn enter_inner(pagedir: usize, old_proc_esp: *const RwLock<usize>, new_proc_esp: *const RwLock<usize>) {
  llvm_asm!("mov cr3, $0" : : "r"(pagedir) : : "intel", "volatile");
  let cur_esp;
  llvm_asm!("mov $0, esp" : "=r"(cur_esp) : : : "intel", "volatile");
  *(*old_proc_esp).write() = cur_esp;
  let next_esp = (*new_proc_esp).read().clone();
  llvm_asm!("mov esp, $0; iretd" : : "r"(next_esp) : : "intel", "volatile");
}

pub fn yield_coop() {
  let next = all_processes().get_next_running_process();
  let current_pid = all_processes().get_current_pid();
  if next != current_pid {
    switch_to(next);
  }
}

pub fn sleep(ms: usize) {
  all_processes().get_current_process().unwrap().sleep(ms);
  yield_coop();
}

pub fn send_tick() {
  for (_id, p) in all_processes().iter() {
    p.update_tick();
  }
}

pub extern "C" fn fork() -> u32 {
  unsafe {
    llvm_asm!("push ebx; push ebp; push esi; push edi" : : : "esp" : "intel", "volatile");
    let ret;
    {
      let next_pid = all_processes_mut().fork_current();
      llvm_asm!("push $0" : : "r"(next_pid.as_u32()) : "esp" : "intel", "volatile");
      
      let stack_container = {
        let processes = all_processes();
        let next_proc = processes.get_process(next_pid).unwrap();
        next_proc.make_current_stack_frame_editable();

        let temp_page_address = crate::memory::virt::page_directory::get_temporary_page_address().as_usize();
        let kernel_esp: usize;
        llvm_asm!("mov $0, esp" : "=r"(kernel_esp) : : : "intel", "volatile");
        let stack_offset = kernel_esp & 0xfff;

        let stack_ptr = (temp_page_address + stack_offset) as *mut usize;
        // Update the return value for the new thread
        *stack_ptr = 0;
        next_proc.get_kernel_stack_container() as *const RwLock<usize>
      };
      fork_inner(stack_container);
    }
    llvm_asm!("pop $0" : "=r"(ret) : : "esp" : "intel", "volatile");
    llvm_asm!("pop edi; pop esi; pop ebp; pop ebx" : : : "esp", "edi", "esi", "ebp", "ebx" : "intel", "volatile");
    ret
  }
}

#[naked]
#[inline(never)]
unsafe fn fork_inner(new_proc_esp: *const RwLock<usize>) {
  let cur_esp;
  llvm_asm!("mov $0, esp" : "=r"(cur_esp) : : : "intel", "volatile");
  // This is super hacky, but it's what we get for making the stack copied
  // directly rather than copy-on-write
  llvm_asm!("mov eax, esp; and eax, 0xfff; add eax, 0xffbff000; mov ecx, [esp]; mov [eax], ecx" : : : "eax", "ecx" : "intel", "volatile");
  *(*new_proc_esp).write() = cur_esp;
}
