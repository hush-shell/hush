use std::os::unix::prelude::{AsRawFd, RawFd};


pub type FileDescriptor = RawFd;


pub fn stdout_fd() -> FileDescriptor {
	std::io::stdout().as_raw_fd()
}
