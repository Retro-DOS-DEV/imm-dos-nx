/// A vterm virtualizes access to the keyboard input and video output.
/// This is how the operating system achieves multitasking from the user's
/// perspective. DOS is inherently a single-tasking environment, where each
/// program takes over the entire screen. By capturing keyboard hooks to switch
/// between environments, it allows the user to run multiple DOS applications in
/// parallel.
/// 
/// Switching requires that each vterm stores all state necessary to reconstruct
/// the video state at any time, and can track any changes that happen while
/// inactive.
pub struct VTerm {
  pub video_mode: u8,
  
}

impl VTerm {

}