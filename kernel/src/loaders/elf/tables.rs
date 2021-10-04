#[repr(C, packed)]
pub struct Header {
  magic_number: [u8; 4],
  /// 1 indicates 32-bit, 2 indicates 64-bit
  bit_class: u8,
  /// 1 indicates little-endian, 2 indicates big-endian format for all header fields
  endian: u8,
  identifier_version: u8,
  /// Target OS ABI, but commonly set to zero
  target_abi: u8,
  abi_version: u8,
  padding: [u8; 7],
  /// Determines how this object file should be interpreted
  object_file_type: u16,
  /// Indicates the target machine architecture
  machine: u16,
  elf_version: u32,
  /// Address of the program entry point
  pub entry_point: u32,
  /// Pointer to the program header table, as an offset from the file start
  pub program_header_table_offset: u32,
  /// Pointer to the section header table, as an offset from the file start
  pub section_header_table_offset: u32,
  flags: u32,
  /// Size of this header, typically 52 bytes
  pub header_size: u16,
  /// Size of a single entry in the program header table
  pub program_header_table_entry_size: u16,
  /// Count of entries in the program header table
  pub program_header_table_count: u16,
  /// Size of a single entry in the section header table
  pub section_header_table_entry_size: u16,
  /// Count of entries in the section header table
  pub section_header_table_count: u16,
  /// Index of the section table entry that has section names
  pub section_header_strings_index: u16,
}

#[repr(C, packed)]
pub struct ProgramHeader {
  pub segment_type: u32,
  pub segment_file_offset: u32,
  pub segment_virtual_address: u32,
  pub segment_physical_address: u32,
  pub segment_size_in_file: u32,
  pub segment_size_in_memory: u32,
  pub segment_flags: u32,
  pub segment_alignment: u32,
}

pub const SEGMENT_FLAG_EXECUTE: u32 = 1;
pub const SEGMENT_FLAG_WRITE: u32 = 2;
pub const SEGMENT_FLAG_READ: u32 = 4;

pub const SEGMENT_TYPE_NULL: u32 = 0;
pub const SEGMENT_TYPE_LOAD: u32 = 1;
pub const SEGMENT_TYPE_DYNAMIC: u32 = 2;

#[repr(C, packed)]
pub struct SectionHeader {
  /// Offset to this section's name in the string table
  pub section_name_offset: u32,
  pub section_type: u32,
  pub section_flags: u32,
  pub section_virtual_address: u32,
  pub section_file_offset: u32,
  pub section_size_in_file: u32,
  pub section_link: u32,
  pub section_info: u32,
  pub section_alignment: u32,
  pub section_entry_size: u32,
}

/// Inaction section header - the first entry in the table is always null
pub const SECTION_TYPE_NULL: u32 = 0;
/// Bits defined by the program, which have no meaning during load / interp
pub const SECTION_TYPE_PROGBITS: u32 = 1;
/// Symbol table
pub const SECTION_TYPE_SYMTAB: u32 = 2;
/// String table
pub const SECTION_TYPE_STRTAB: u32 = 3;
/// Relocations with known "addends"
pub const SECTION_TYPE_RELA: u32 = 4;
/// Hash table for symbol lookup
pub const SECTION_TYPE_HASH: u32 = 5;
/// Information needed for dynamic linking
pub const SECTION_TYPE_DYNAMIC: u32 = 6;
/// Markers specific to the file contents
pub const SECTION_TYPE_NOTE: u32 = 7;
/// Not backed by content from the file
pub const SECTION_TYPE_NOBITS: u32 = 8;
/// Relocations without addends
pub const SECTION_TYPE_REL: u32 = 9;


pub const SECTION_FLAG_WRITE: u32 = 1;
pub const SECTION_FLAG_ALLOC: u32 = 2;
pub const SECTION_FLAG_EXEC: u32 = 4;
pub const SECTION_FLAG_MERGE: u32 = 0x10;
pub const SECTION_FLAG_STRINGS: u32 = 0x20;