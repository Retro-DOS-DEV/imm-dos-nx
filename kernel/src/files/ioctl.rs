
const IOC_OUT: u32 = 0x40000000;
const IO_PARAM_MASK: u32 = 0x1fff;

pub const FIONREAD: u32 = IOC_OUT | (4 << 16) | (0x66 << 6) | 0xff;
