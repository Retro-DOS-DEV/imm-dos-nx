use alloc::vec::Vec;

/// SlotList is a growing vector where each element may contain an element.
/// When items are removed, that entry (or "slot") can be reused by the next
/// item that needs to be stored. This creates a collection where items can be
/// added and removed without waste, and external references to indexes can
/// remain stable.
/// This data structure is used in many of the internal concepts where items are
/// indexed by a numeric handle, like filesystems and devices.
pub struct SlotList<T: Sized> {
  slots: Vec<Option<T>>,
}

impl<T: Sized> SlotList<T> {
  pub const fn new() -> SlotList<T> {
    SlotList {
      slots: Vec::new(),
    }
  }

  pub fn with_capacity(capacity: usize) -> SlotList<T> {
    SlotList {
      slots: Vec::with_capacity(capacity),
    }
  }

  pub fn find_empty_slot(&mut self) -> usize {
    let mut found: Option<usize> = None;
    let mut index = 0;
    while index < self.slots.len() && found.is_none() {
      if self.slots[index].is_none() {
        found = Some(index);
      }
      index += 1;
    }
    match found {
      Some(i) => i,
      None => {
        let last = self.slots.len();
        self.slots.push(None);
        last
      },
    }
  }

  pub fn insert(&mut self, item: T) -> usize {
    let index = self.find_empty_slot();
    self.slots[index] = Some(item);
    index
  }

  pub fn get(&self, index: usize) -> Option<&T> {
    let slot = self.slots.get(index)?;
    match slot {
      Some(item) => Some(&item),
      None => None,
    }
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    let slot = self.slots.get_mut(index)?;
    slot.as_mut()
  }

  pub fn remove(&mut self, index: usize) -> Option<T> {
    let entry = self.slots.get_mut(index)?;
    let prev = entry.take();
    prev
  }

  pub fn replace(&mut self, index: usize, item: T) -> Option<T> {
    while self.slots.len() <= index {
      self.slots.push(None);
    }
    let entry = self.slots.get_mut(index).unwrap();
    entry.replace(item)
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.slots.iter().filter_map(|i| i.as_ref())
  }

  pub fn map_in_place<F>(&mut self, f: F)
    where F: Fn(&T) -> Option<T> {
    for i in 0..self.slots.len() {
      if let Some(entry) = self.slots.get_mut(i) {
        *entry = if let Some(content) = entry {
          f(content)
        } else {
          None
        };
      }
    }
  }

  pub fn len(&self) -> usize {
    self.slots.len()
  }
}

impl<T: Clone> Clone for SlotList<T> {
  fn clone(&self) -> Self {
    Self {
      slots: self.slots.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::SlotList;

  #[test]
  fn inserting_items() {
    let mut list: SlotList<u32> = SlotList::with_capacity(3);
    assert_eq!(list.get(1), None);
    assert_eq!(list.insert(20), 0);
    assert_eq!(list.insert(30), 1);
    assert_eq!(list.insert(40), 2);
    assert_eq!(list.get(0), Some(&20));
    assert_eq!(list.get(1), Some(&30));
    assert_eq!(list.get(2), Some(&40));
    assert_eq!(list.get(3), None);
  }

  #[test]
  fn grow_to_fit() {
    let mut list: SlotList<u32> = SlotList::new();
    assert_eq!(list.get(1), None);
    assert_eq!(list.insert(20), 0);
    assert_eq!(list.insert(30), 1);
    assert_eq!(list.insert(40), 2);
    assert_eq!(list.get(0), Some(&20));
    assert_eq!(list.get(1), Some(&30));
    assert_eq!(list.get(2), Some(&40));
    assert_eq!(list.get(3), None);
  }

  #[test]
  fn removing_items() {
    let mut list: SlotList<u32> = SlotList::new();
    list.insert(55);
    list.insert(40);
    list.insert(60);
    assert_eq!(list.remove(1), Some(40));
    assert_eq!(list.get(1), None);
  }

  #[test]
  fn replacing_emptied_items() {
    let mut list: SlotList<u32> = SlotList::new();
    list.insert(11);
    list.insert(22);
    list.insert(33);
    list.remove(0);
    list.remove(1);
    assert_eq!(list.insert(44), 0);
    assert_eq!(list.insert(55), 1);
    assert_eq!(list.insert(66), 3);
  }

  #[test]
  fn replacing_existing_entries() {
    let mut list: SlotList<u32> = SlotList::new();
    list.insert(1);
    list.insert(3);
    list.insert(5);
    assert_eq!(list.replace(0, 10), Some(1));
    assert_eq!(list.replace(4, 12), None);
    assert_eq!(list.get(0), Some(&10));
    assert_eq!(list.get(3), None);
    assert_eq!(list.get(4), Some(&12));
  }

  #[test]
  fn iterator() {
    let mut list: SlotList<u32> = SlotList::new();
    list.insert(1);
    list.insert(2);
    list.insert(1);
    list.insert(3);
    list.insert(1);

    list.remove(1);
    list.remove(3);
    let mut count = 0;
    for x in list.iter() {
      count += 1;
      assert_eq!(*x, 1);
    }
    assert_eq!(count, 3);
  }
}
