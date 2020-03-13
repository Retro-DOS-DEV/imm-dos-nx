use crate::{devices, kprint, time};

pub extern "C" fn pit() {
  let prev = time::get_offset_seconds();
  time::increment_offset(100002);
  let updated = time::get_offset_seconds();
  if prev != updated {
    kprint!("T");
  }

  unsafe {
    devices::PIC.acknowledge_interrupt(0);
  }
}
