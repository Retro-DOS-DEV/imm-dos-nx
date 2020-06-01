pub fn get_cr3() -> u32 {
  let cr3: u32;
  unsafe {
    llvm_asm!("mov $0, cr3" : "=r"(cr3) : : : "intel", "volatile");
  }
  cr3
}

pub fn set_cr3(value: u32) {
  unsafe {
    llvm_asm!("mov cr3, $0" : : "r"(value) : : "intel", "volatile");
  }
}

pub fn enable_paging() {
  unsafe {
    llvm_asm!("mov eax, cr0
          or eax, 0x80000000
          mov cr0, eax" : : :
          "eax" :
          "intel", "volatile"
    );
  }
}