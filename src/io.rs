use std::os::unix::prelude::{AsRawFd, RawFd};


pub type FileDescriptor = RawFd;


/// Get the file descriptor for stdout.
pub fn stdout_fd() -> FileDescriptor {
	std::io::stdout().as_raw_fd()
}
