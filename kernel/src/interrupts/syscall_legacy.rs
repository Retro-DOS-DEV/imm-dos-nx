use crate::dos::{
  devices,
  execution,
  registers::{DosApiRegisters, VM86Frame}
};
use crate::kprintln;
use super::stack::StackFrame;

/**
 * Interrupts to support legacy DOS API calls
 */
pub fn dos_api(regs: &mut DosApiRegisters, segments: &mut VM86Frame, stack_frame: &StackFrame) {
  match (regs.ax & 0xff00) >> 8 {
    0 => { // Terminate
      let new_address = execution::terminate(stack_frame.cs as u16);
      unsafe {
        stack_frame.set_eip(new_address.offset as u32);
        stack_frame.set_cs(new_address.segment as u32);
      }
    },
    1 => { // Keyboard input with Echo
      devices::read_stdin_with_echo(regs);
    },
    0x02 => { // Print character to STDOUT
      // Debugging body
      kprintln!("PRINTDOS");
      regs.ax = (regs.ax & 0xff00) | (regs.dx & 0xff);
    },
    0x03 => { // Wait for STDAUX
      // Blocks until a character can be read from STDAUX
    },
    0x04 => { // Output to AUX
      // Send a single character to STDAUX
    },
    0x05 => { // Output to Printer
      // Send a single character to STDPRN
    },
    0x06 => { // Console IO
      let output = regs.dx & 0xff;
      if output == 0xff { // input request
        // Read raw keyboard input, without echo
        // If a character is available, copy it to %al and clear the zero flag
        // If no character is available, set the zero flag
      } else { // output request
        // Write character to console
      }
    },
    0x07 => { // Blocking console input
      // Block until character is available from the keyboard
    },
    0x08 => { // Blocking STDIN input
      // Block until character is available from STDIN
    },
    0x09 => { // Print string
      // DS:DX points to a string terminated by '$'
      // Copy the string to STDOUT
      devices::print_string(regs, segments);
    },
    0x0a => { // Buffered keyboard input
      // Used to read multiple characters at a time
      // DS:DX points to a buffer in the following format:
      // | max to read | chars read | buffer [...]
      // Max buffer size is 255 bytes, since the first two values are each one
      // byte long.
      // If a CR is encountered in the input stream, it will be copied to the
      // buffer, and then abort.
    },
    0x0b => { // Check STDIN readiness
      // Set %al to 0 if nothing is available
      // Set %al to 0xff if a character is available
    },
    0x0c => { // Clear keyboard buffer, run a keyboard function
    },
    0x0d => { // Reset disk
      // Flush all file buffers to disk
    },
    0x0e => { // Select disk
      // Set the drive letter for the "active" disk
      // %dl is zero-based, 0 == A:, 25 == Z:
      let drive = regs.dx & 0xff;
      // On return, set %al to the letter of available drives
    },
    0x0f => { // Open file using FCB
      // DS:DX points to a FCB
      // The FCB has some fields set to determine where the file is
      // Once the file is open, the other fields in the FCB are set
      // If successful, %al is set to 0. Otherwise, set to 0xff.
    },
    0x10 => { // Close file using FCB
      // DS:DX points to a FCB
      // If successful, %al is set to 0. Otherwise, set to 0xff.
    },
    0x11 => { // Search for first match using FCB
      // DS:DX points to a FCB containing search parameters
      // Searching does not overwrite the requesting FCB. Instead, it writes to
      // the process's DTA location.
      // To perform this, we open a directory handle to the CWD, and iterate
      // through all entries until we find a match.
      // If a match is found, %al is set to 0, otherwise set to 0xff.
    },
    0x12 => { // Search for next match using FCB
      // Must be performed after 0x11
      // DS:DX points to a FCB containing search parameters
      // Uses the DTA entry from the previous 0x11 call to find the prev match,
      // then continues the search for the next matching entry.
    },
    0x13 => { // Delete file using FCB
      // DS:DX points to a FCB
      // If successful, %al is set to 0. Otherwise, set to 0xff.
    },
    0x14 => { // Sequential read using FCB
      // Read a single record of data from an open FCB
      // DS:DX points to a FCB
      // Data is copied to the DTA
    },
    0x15 => { // Sequential write using FCB
      // Write a single record of data to an open FCB
      // DS:DX points to a FCB
      // Data is copied from the DTA
    },
    0x16 => { // Create file using FCB
    },
    0x17 => { // Rename file using FCB
      // DS:DX points to a "custom" FCB with the following offsets:
      //    0 => Drive number
      //    1 => Original filename
      //    9 => Original extension
      //   11 => New filename
      //   19 => New extension
    },
    0x18 => { // Dummy function
    },
    0x19 => { // Get current drive
      // Set %al to the zero-based number representing the current drive
    },
    0x1a => { // Set DTA
      // DS:DX contains the address to the new DTA location
    },
    0x1b => { // Get FAT info for the current drive
      // Set %al to sectors per cluster
      // Set %cx to bytes per sector
      // Set %dx to clusters on disk
      // Set DS:BX to the media descriptor type
    },
    0x1c => { // Get FAT info for a specific drive
      // Same as 0x1b, but %dl contains the drive to fetch info from
    },
    0x1d => { // Dummy function
    },
    0x1e => { // Dummy function
    },
    0x1f => { // Get pointer to drive parameter table
    },
    0x20 => { // Dummy function
    },
    0x21 => { // Random read using FCB
      // Read a record from disk without updating the cursor in the FCB
    },
    0x22 => { // Random write using FCB
      // Write a record to disk without updating the cursor in the FCB
    },
    0x23 => { // Get file size using FCB
      // DS:DX points to a FCB
      // Open the file, and set the random record position to the total record
      // count.
    },
    0x24 => { // Update relative record field in FCB
      // Set the random record field to the current sequential field
    },
    0x25 => { // Set an interrupt vector
    },
    0x26 => { // Create new PSP
      // Allocates a new PSP after the current program, and copies the current
      // PSP to that location.
    },
    0x27 => { // Random block read using FCB
    },
    0x28 => { // Random block write using FCB
    },
    0x29 => { // Parse filename for FCB use
    },
    0x2a => { // Get date
    },
    0x2b => { // Set date
    },
    0x2c => { // Get time
    },
    0x2d => { // Set time
    },
    0x2e => { // Set disk verification mode
    },
    0x2f => { // Get DTA
    },
    0x30 => { // Get DOS Version
    },
    0x31 => { // Terminate and Stay Resident
    },
    0x32 => { // Get pointer to specified drive param table
    },
    0x33 => { // Update ctrl-break checking
    },
    0x34 => { // Get address for critical flag
    },
    0x35 => { // Get interrupt vector
    },
    0x36 => { // Get free space
    },
    0x37 => { // Update switch character
    },
    0x38 => { // Locale-dependent info
    },
    0x39 => { // mkdir
    },
    0x3a => { // rmdir
    },
    0x3b => { // chdir
    },
    0x3c => { // Create file using handle
    },
    0x3d => { // Open file using handle
    },
    0x3e => { // Close file using handle
    },
    0x3f => { // Read file using handle
    },
    0x40 => { // Write file using handle
    },
    0x41 => { // Delete file
    },
    0x42 => { // Move file pointer using handle
    },
    0x43 => { // Change file mode
    },
    0x44 => { // IOCTL
    },
    0x45 => { // Dup file handle
    },
    0x46 => { // Force dup file handle
    },
    0x47 => { // Get cwd
    },
    0x48 => { // Allocate memory
    },
    0x49 => { // Free memory
    },
    0x4a => { // Modify allocated memory
    },
    0x4b => { // Load and execute program
    },
    0x4c => { // Terminate with return code
    },
    0x4d => { // Get return code of child
    },
    0x4e => { // Find first matching file
    },
    0x4f => { // Find next matching file
    },
    0x50 => { // Set current PSP
    },
    0x51 => { // Get current PSP
    },
    0x52 => { // Get INVARS
    },
    0x53 => { // Create drive param table
    },
    0x54 => { // Get disk verify setting
    },
    0x55 => { // Create PSP
    },
    0x56 => { // Rename file
    },
    0x57 => { // Read/Write file datetime
    },
    0x58 => { // Modify memory allocation strategy
    },
    0x59 => { // Get error info
    },
    0x5a => { // Create temp file
    },
    0x5b => { // Create new file
    },

    _ => (),
  }
}