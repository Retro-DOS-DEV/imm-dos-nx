#[cfg(not(test))]
#[inline(never)]
pub extern fn cleanup_process() {
  use alloc::vec::Vec;
  use crate::task::id::ProcessID;

  crate::kprintln!("Cleanup process ready");

  let mut terminated: Vec<ProcessID> = Vec::new();

  loop {
    crate::task::switching::for_each_process_mut(|p| {
      let process = p.read();
      if process.is_terminated() {
        terminated.push(*process.get_id());
      }
    });

    while !terminated.is_empty() {
      if let Some(id) = terminated.pop() {
        crate::task::switching::clean_up_process(id);
      }
    }

    crate::task::yield_coop();
  }
}