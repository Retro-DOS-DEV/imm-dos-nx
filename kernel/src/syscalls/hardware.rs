pub fn change_video_mode(mode: u32) {
  crate::hardware::vga::driver::request_mode_change(mode);
}