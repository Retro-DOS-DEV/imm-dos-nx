extern crate cc;
use std::env;

fn main() {
  // Only include assembly libraries when we're building the kernel.
  // The assembly may not be supported on the host machine.
  if env::var("CARGO_CFG_TARGET_OS").unwrap().eq(&String::from("none")) {
    cc::Build::new()
      .flag("-m32")
      .flag("-march=i386")
      .file("src/asm/syscall.s")
      .compile("libsyscall");
    
    cc::Build::new()
      .flag("-m32")
      .flag("-march=i386")
      .file("src/asm/irq.s")
      .compile("libirq");
  }
}