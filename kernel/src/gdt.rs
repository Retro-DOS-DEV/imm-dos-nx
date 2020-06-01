use core::mem;

pub const GDT_ACCESS_PRESENT: u8 = 1 << 7;
pub const GDT_ACCESS_RING_0: u8 = 0;
pub const GDT_ACCESS_RING_1: u8 = 1 << 5;
pub const GDT_ACCESS_RING_2: u8 = 2 << 5;
pub const GDT_ACCESS_RING_3: u8 = 3 << 5;
pub const GDT_ACCESS_CODE_DATA_DESCRIPTOR: u8 = 1 << 4;
pub const GDT_ACCESS_SYSTEM_DESCRIPTOR: u8 = 0;
pub const GDT_ACCESS_EXECUTABLE: u8 = 1 << 3;
pub const GDT_ACCESS_CONFORMING: u8 = 1 << 2;
pub const GDT_ACCESS_GROW_DOWN: u8 = 1 << 2;
pub const GDT_ACCESS_RW: u8 = 1 << 1;
pub const GDT_ACCESS_ACCESSED: u8 = 1;

pub const GDT_FLAG_GRANULARITY_4KB: u8 = 1 << 7;
pub const GDT_FLAG_GRANULARITY_1B: u8 = 0;
pub const GDT_FLAG_SIZE_32_BIT: u8 = 1 << 6;
pub const GDT_FLAG_SIZE_16_BIT: u8 = 0;

#[repr(C, packed)]
pub struct GDTEntry {
  pub limit_low: u16,
  pub base_low: u16,
  pub base_middle: u8,
  pub access: u8,
  pub flags_and_limit_high: u8,
  pub base_high: u8,
}

impl GDTEntry {
  pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> GDTEntry {
    GDTEntry {
      limit_low: (limit & 0xffff) as u16,
      base_low: (base & 0xffff) as u16,
      base_middle: ((base >> 16) & 0xff) as u8,
      access,
      flags_and_limit_high: (flags & 0xe0) | (((limit >> 16) & 0xf) as u8),
      base_high: ((base >> 24) & 0xff) as u8,
    }
  }

  pub fn set_base(&mut self, base: u32) {
    self.base_low = (base & 0xffff) as u16;
    self.base_middle = ((base >> 16) & 0xff) as u8;
    self.base_high = ((base >> 24) & 0xff) as u8;
  }

  pub fn set_limit(&mut self, limit: u32) {
    self.limit_low = (limit & 0xffff) as u16;
    self.flags_and_limit_high = (self.flags_and_limit_high & 0xf0) | (((limit >> 16) & 0xf) as u8);
  }
}

#[repr(C, packed)]
pub struct GDTDescriptor {
  pub size: u16,
  pub offset: u32,
}

pub unsafe fn lgdt(desc: &GDTDescriptor) {
  llvm_asm!("lgdt [$0]" : : "r"(desc) : : "intel", "volatile");
}

pub unsafe fn ltr(index: u16) {
  let selector = index | 3;
  llvm_asm!("ltr $0" : : "r"(selector) : : "intel", "volatile");
}

// Global tables:

static mut GDTR: GDTDescriptor = GDTDescriptor {
  size: 0,
  offset: 0,
};

static mut GDT: [GDTEntry; 6] = [
  // Null entry - 0x00
  GDTEntry::new(0, 0, 0, 0),

  // Kernel code - 0x08
  GDTEntry::new(
    0,
    0xffffffff,
    GDT_ACCESS_PRESENT | GDT_ACCESS_RING_0 | GDT_ACCESS_CODE_DATA_DESCRIPTOR | GDT_ACCESS_EXECUTABLE | GDT_ACCESS_RW,
    GDT_FLAG_GRANULARITY_4KB | GDT_FLAG_SIZE_32_BIT
  ),

  // Kernel data - 0x10
  GDTEntry::new(
    0,
    0xffffffff,
    GDT_ACCESS_PRESENT | GDT_ACCESS_RING_0 | GDT_ACCESS_CODE_DATA_DESCRIPTOR | GDT_ACCESS_RW,
    GDT_FLAG_GRANULARITY_4KB | GDT_FLAG_SIZE_32_BIT
  ),

  // User code - 0x18
  GDTEntry::new(
    0,
    0xffffffff,
    GDT_ACCESS_PRESENT | GDT_ACCESS_RING_3 | GDT_ACCESS_CODE_DATA_DESCRIPTOR | GDT_ACCESS_EXECUTABLE | GDT_ACCESS_RW,
    GDT_FLAG_GRANULARITY_4KB | GDT_FLAG_SIZE_32_BIT
  ),

  // User data - 0x20
  GDTEntry::new(
    0,
    0xffffffff,
    GDT_ACCESS_PRESENT | GDT_ACCESS_RING_3 | GDT_ACCESS_CODE_DATA_DESCRIPTOR | GDT_ACCESS_RW,
    GDT_FLAG_GRANULARITY_4KB | GDT_FLAG_SIZE_32_BIT
  ),

  // TSS - 0x28
  GDTEntry::new(
    0,
    0xffffffff,
    GDT_ACCESS_PRESENT | GDT_ACCESS_RING_3 | GDT_ACCESS_SYSTEM_DESCRIPTOR | GDT_ACCESS_EXECUTABLE | GDT_ACCESS_ACCESSED,
    0
  ),
];

#[repr(C, packed)]
pub struct TaskStateSegment {
  prev_tss: u32,
  esp0: u32,
  ss0: u32,
  esp1: u32,
  ss1: u32,
  esp2: u32,
  ss2: u32,
  cr3: u32,
  eip: u32,
  eflags: u32,
  eax: u32,
  ecx: u32,
  edx: u32,
  ebx: u32,
  esp: u32,
  ebp: u32,
  esi: u32,
  edi: u32,
  es: u32,
  cs: u32,
  ss: u32,
  ds: u32,
  fs: u32,
  gs: u32,
  ldt: u32,
  trap: u16,
  iomap_base: u16,
}

impl TaskStateSegment {
  pub fn zero(&mut self) {
    self.prev_tss = 0;
    self.esp0 = 0;
    self.ss0 = 0;
    self.esp1 = 0;
    self.ss1 = 0;
    self.esp2 = 0;
    self.ss2 = 0;
    self.cr3 = 0;
    self.eip = 0;
    self.eflags = 0;
    self.eax = 0;
    self.ecx = 0;
    self.edx = 0;
    self.ebx = 0;
    self.esp = 0;
    self.ebp = 0;
    self.esi = 0;
    self.edi = 0;
    self.es = 0;
    self.cs = 0;
    self.ss = 0;
    self.ds = 0;
    self.fs = 0;
    self.gs = 0;
    self.ldt = 0;
    self.trap = 0;
    self.iomap_base = 0;
  }

  pub fn set_stack_segment(&mut self, segment: u32) {
    self.ss0 = segment;
  }

  pub fn set_stack_pointer(&mut self, pointer: u32) {
    self.esp0 = pointer;
  }
}

static mut TSS: TaskStateSegment = TaskStateSegment {
  prev_tss: 0,
  esp0: 0,
  ss0: 0,
  esp1: 0,
  ss1: 0,
  esp2: 0,
  ss2: 0,
  cr3: 0,
  eip: 0,
  eflags: 0,
  eax: 0,
  ecx: 0,
  edx: 0,
  ebx: 0,
  esp: 0,
  ebp: 0,
  esi: 0,
  edi: 0,
  es: 0,
  cs: 0,
  ss: 0,
  ds: 0,
  fs: 0,
  gs: 0,
  ldt: 0,
  trap: 0,
  iomap_base: 0,
};

pub unsafe fn init() {
  GDTR.size = (GDT.len() * mem::size_of::<GDTEntry>() - 1) as u16;
  GDTR.offset = GDT.as_ptr() as *const GDTEntry as u32;

  TSS.zero();
  TSS.set_stack_segment(0x10);
  GDT[5].set_limit(mem::size_of::<TaskStateSegment>() as u32);
  GDT[5].set_base(&TSS as *const TaskStateSegment as u32);

  lgdt(&GDTR);
  ltr(0x28);
}

pub unsafe fn set_tss_stack_pointer(sp: u32) {
  TSS.set_stack_pointer(sp);
}
