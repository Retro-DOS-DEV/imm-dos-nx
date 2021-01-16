//! Parsing and loading for DOS MZ EXE files

#[repr(C, packed)]
pub struct MZHeader {
  magic_number: [u8; 2],
  /// Number of bytes actually occupied in the final page
  last_page_size: u16,
  /// Number of 512B pages needed to contain this file
  page_count: u16,
  /// Number of entries in the relocation table
  relocation_entries: u16,
  /// Size of this header, in paragraphs (4 bytes)
  header_size_paragraphs: u16,
  /// Minimum number of paragraphs required for execution. This is used for
  /// uninitialized data that appears
  min_alloc_paragraphs: u16,
  /// Maximum number of paragraphs required for execution; this is the amount
  /// preferred by the program.
  max_alloc_paragraphs: u16,
  /// Initial value of the SS segment, added to the program's first segment
  initial_ss: u16,
  /// Initial value of the SP register
  initial_sp: u16,
  /// Data integrity checksum
  checksum: u16,
  /// Initial value of the IP register
  initial_ip: u16,
  /// Initial value of the CS segment, added to the program's first segment
  initial_cs: u16,
  /// Location of the relocation table, relative to the start of the file
  relocation_table_offset: u16,
  /// Overlay number (wut?)
  overlay_number: u16,
}

impl MZHeader {
  pub fn byte_length(&self) -> usize {
    if self.page_count == 0 {
      return 0;
    }
    (self.page_count as usize - 1) * 512 + (self.last_page_size as usize)
  }
}