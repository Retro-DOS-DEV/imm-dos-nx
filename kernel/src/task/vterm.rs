pub fn get_current_vterm() -> Option<usize> {
  let current_process_lock = super::get_current_process();
  let current_process = current_process_lock.read();
  current_process.get_vterm()
}