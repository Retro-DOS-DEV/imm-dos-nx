#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
  pub const fn new(index: u16, priv_level: u16) -> SegmentSelector {
    SegmentSelector(
      (index << 3) |
      (priv_level & 3)
    )
  }
}