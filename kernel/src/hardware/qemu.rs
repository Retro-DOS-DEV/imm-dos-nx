use crate::x86::io::Port;

const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;

pub fn debug_exit(code: u32) {
  let port = Port::new(ISA_DEBUG_EXIT_PORT);
  unsafe { port.write_u32(code) };
}