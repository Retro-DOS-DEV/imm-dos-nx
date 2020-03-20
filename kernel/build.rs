extern crate cc;

fn main() {
  cc::Build::new()
    .flag("-m32")
    .flag("-march=i386")
    .file("src/asm/syscall.s")
    .compile("libsyscall");
}