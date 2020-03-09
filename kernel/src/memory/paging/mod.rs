use super::address::{PhysicalAddress, VirtualAddress};
use super::frame::Frame;

pub mod directory;
pub mod entry;
pub mod table;

pub fn map_address_to_frame(vaddr: VirtualAddress, frame: Frame) {
  let mut dir = directory::get_current_directory();
  let dir_entry = dir.get_entry_for_address(vaddr);
  let dir_index = vaddr.as_usize() >> 22;
  let table_address = if dir_entry.is_present() {
    dir_entry.get_address()
  } else {
    // need to allocate a frame for a new page table
    let table_frame: Frame = match super::allocate_physical_frame() {
      Ok(f) => f,
      Err(_) => {
        panic!("Failed to allocate memory for a new page table")
      },
    };
    let paddr = table_frame.get_address();
    dir.set_table_at_entry(paddr, dir_index);
    paddr
  };
  // All table entries are mapped at the last 4MiB of memory
  // Table entry n is at 0xffc00000 + 4096 * n
  let location = VirtualAddress::new(dir_index * 4096 + 0xffc00000);
  let mut page_table = table::PageTable {
    physical_location: table_address,
    virtual_location: location,
  };
  let mut entry = page_table.get_entry_for_address(vaddr);
  if entry.is_present() {
    // IDK, do we do something here?
  }
  entry.set_address(frame.get_address());
  entry.set_present();
  page_table.set_table_entry((vaddr.as_usize() >> 12) & 0x3ff, entry);
}

pub fn get_mapping(vaddr: VirtualAddress) -> Option<PhysicalAddress> {
  let dir = directory::get_current_directory();
  let table_entry = dir.get_entry_for_address(vaddr);
  

  None
}