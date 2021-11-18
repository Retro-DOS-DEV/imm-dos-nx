pub fn change_video_mode(mode: u8) {
  let vterm_index = match crate::task::vterm::get_current_vterm() {
    Some(current) => current,
    None => return,
  };
  crate::vterm::change_video_mode(vterm_index, mode);
}
