/// Reference to a sector, in LBA format
#[derive(Copy, Clone)]
pub struct Sector(usize);

const SECTORS_PER_TRACK: usize = 18;
const SECTOR_SIZE: usize = 512;

impl Sector {
  pub fn to_chs(&self) -> (usize, usize, usize) {
    let c = self.0 / (2 * SECTORS_PER_TRACK);
    let h = (self.0 % (2 * SECTORS_PER_TRACK)) / SECTORS_PER_TRACK;
    let s = (self.0 % (2 * SECTORS_PER_TRACK)) % SECTORS_PER_TRACK + 1;
    (c, h, s)
  }
}

pub struct SectorRange {
  first: Sector,
  count: usize,
}

impl SectorRange {
  pub fn for_byte_range(start: usize, length: usize) -> SectorRange {
    let sector_start = start & !(SECTOR_SIZE - 1);
    let range_end = start + length;
    let mut sector_count = (range_end - sector_start) / SECTOR_SIZE;
    if range_end & (SECTOR_SIZE - 1) != 0 {
      sector_count += 1;
    }
    SectorRange {
      first: Sector(sector_start / SECTOR_SIZE),
      count: sector_count,
    }
  }

  pub fn byte_length(&self) -> usize {
    self.count * SECTOR_SIZE
  }

  pub fn get_first_sector(&self) -> Sector {
    self.first
  }

  pub fn get_local_offset(&self, absolute: usize) -> usize {
    let start = self.first.0 * SECTOR_SIZE;
    if absolute < start {
      0
    } else {
      absolute - start
    }
  }
}
