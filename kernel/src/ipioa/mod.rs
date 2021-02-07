//! IPIOA - Inter-Process I/O Arbiter
//! The IPIOA is a kernel service that allows device driver and filesystems
//! to be implemented in userspace. It runs in a loop, processing incoming IPC
//! messages and creating requests that get distributed to different drivers.
//! 
//! When a filesystem or driver request is made to a device that isn't compiled
//! into the kernel, a message for that request is sent to the IPIOA. The
//! message tells the service what kind of request was made, what driver to
//! forward it to, and where to find any associated data like a read/write
//! buffer. The service creates a request object and enqueues it for the driver.
//! Only one request can be sent to a driver at a time, so the arbiter stores up
//! any other requests until the current one is completed. The service then
//! returns to reading its IPC message queue.
//! The driver is eventually sent an IPC request to make it aware of the new
//! request. It performs any processing necessary to fulfill that request, and
//! sends a message back to the arbiter telling it where to find the results.
//! When the arbiter receives a completion message, it looks up the associated
//! request to determine what to do result (and if it's even still needed -- the
//! requestor could have aborted it or exited). As a kernel service, it performs
//! any necessary process-to-process copying, and awakens the original caller.
//! 
//! Because filesystems and device drivers perform similar operations, and can
//! both be built on the same message-passing infrastructure, a single arbiter
//! process can handle requests to both.
//! 
//! From the Driver's Perspective:
//! Drivers should also be implemented as loops that block on IPC messages.
//! When a message comes from the arbiter to initiate a request, it should reset
//! all state and begin the request. Some operations like writing to a driver
//! are composed of multiple messages, and if the driver receives an initiating
//! message while expecting a secondary step, it should treat the original
//! request as aborted.
//! Once a request is initiated, the driver performs some async actions,
//! blocking on interrupts and external signals as necessary. When completed, it
//! sends the appropriate response to the arbiter, and awaits the next incoming
//! operation.
//! 
//! For both the arbiter and the driver, messages can be "authenticated" by
//! checking the highest bit in the first message argument. This is set on all
//! IPC requests that are sent from kernel-space code.

use crate::task::switching;

pub fn ipioa_run() {
  // Perform setup

  // Run the event loop
  loop {
    // Block on incoming messages
    
  }
}