pub mod address;
pub mod map;
pub mod physical;
pub mod virt;

// not test-safe
#[cfg(not(test))]
pub mod heap;

/// Move the instruction pointer to the high kernel addresses above 0xC0000000.
/// If we don't do this, many of the pointers stored in memory will be incorrect
/// when we later unmap the lower copy of the kernel.
#[cfg(not(test))]
#[naked]
#[inline(never)]
pub unsafe extern "C" fn high_jump() {
  asm!(
    "mov eax, [esp]
    or eax, 0xc0000000
    mov [esp], eax
    ret",
    options(noreturn),
  );
}
