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
