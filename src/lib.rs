//! Cross-platform named-pipe API.
//!
//! # Quick Start
//!
//! To get started quickly, try using Pipe::with_name to create a pipe with a 
//! given name.
//! ```
//! use ipipe::{Pipe, OnCleanup};
//! fn main() -> ipipe::Result<()>
//! {
//!     let mut pipe = Pipe::with_name("test_pipe", OnCleanup::Delete)?;
//!     println!("Pipe path: {}", pipe.path().display());
//!
//!     // Read a line
//!     println!("{}", pipe.read_string_while(|c| c != '\n').unwrap());
//!     Ok(())
//! }
//! ```
//! 
//! Then in another program:
//! ```
//! fn main() -> ipipe::Result<()>
//! {
//!     let mut pipe = Pipe::with_name("test_pipe", OnCleanup::Delete)?;
//!     pipe.write_string("This is only a test.\n")?;
//! }
//! ```
//! You can also use `Pipe::create` to open a pipe with a randomly-generated
//! name, which can then be accessed by calling Pipe::path.
//! 
//! Lastly, Pipe::open can be used to specify an exact path. This is not
//! platform agnostic, however, as Windows pipe paths require a special
//! format.
//!
//! Calling `clone()` on a pipe will create a slave instance. Slave instances 
//! will not delete or close the pipe when they go out of scope. This allows
//! readers and writers to the same pipe to be passed to different threads and 
//! contexts.

#[cfg(unix)]
mod fifo_unix;
#[cfg(unix)]
pub use fifo_unix::*;

#[cfg(windows)]
mod fifo_windows;
#[cfg(windows)]
pub use fifo_windows::*;

#[cfg(feature="static_pipe")]
#[macro_use]
mod static_pipe;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub enum OnCleanup
{
    Delete,
    NoDelete
}

/// Iterator over bytes from the pipe
pub struct FifoIterator<'a>(&'a mut Pipe);
impl Iterator for FifoIterator<'_>
{
    type Item = u8;

    fn next(&mut self) -> Option<u8> 
    {
        if self.0.is_closed
        {
            None
        }
        else
        {
            match self.0.read_byte()
            {
                Ok(byte) => Some(byte),
                Err(err) => 
                {
                    eprintln!("{:?}", err);
                    None
                }
            }
        }
    }
}

impl Pipe
{
    /// Return the path to this named pipe
    pub fn path(&self) -> &std::path::Path
    {
        &self.path
    }

    /// Creates an iterator that reads bytes until the pipe is closed.
    pub fn iter(&mut self) -> FifoIterator
    {
        FifoIterator(self)
    }

    /// Reads until the given predicate is false, and returns the result as a 
    /// vector of bytes.
    pub fn read_bytes_while(&mut self, predicate: impl Fn(u8) -> bool) -> Result<Vec<u8>>
    {
        let mut buf: Vec<u8> = Vec::new();
        loop
        {
            let byte = self.read_byte()?;
            if predicate(byte)
            {
                buf.push(byte);
            } 
            else
            {
                break Ok(buf)
            }
        }
    }

    /// Reads until the given predicate is false, and returns the result as a 
    /// string.
    pub fn read_string_while(&mut self, predicate: impl Fn(u8) -> bool) -> Result<String>
    {
        let mut buf: Vec<u8> = Vec::new();
        loop
        {
            let byte = self.read_byte()?;
            if predicate(byte)
            {
                buf.push(byte);
            } 
            else
            {
                break String::from_utf8(buf).map_err(Error::from)
            }
        }
    }
}

impl std::io::Write for Pipe
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> 
    {
        self.write_bytes(bytes).map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        self.flush_pipe().map_err(std::io::Error::from)
    }
}

/// Standard error type used by this library
#[derive(Debug)]
pub enum Error
{
    InvalidPath,
    InvalidUtf8,
    Io(std::io::Error),
    Native(&'static str, u32, String)
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self
        {
            Error::InvalidPath => write!(f, "Invalid path"),
            Error::InvalidUtf8 => write!(f, "Invalid Utf8"),
            Error::Io(err) => err.fmt(f),
            Error::Native(text, code, oss) => write!(f, "{}: {} - {}", text, code, oss)
        }
    }
}
impl std::error::Error for Error{}

impl From<Error> for std::io::Error
{
    fn from(err: Error) -> std::io::Error
    {
        match err
        {
            Error::Io(err) => err,
            e => std::io::Error::new(std::io::ErrorKind::Other, e)
        }
    }
}

impl From<std::io::Error> for Error
{
    fn from(err: std::io::Error) -> Error
    {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error
{
    fn from(_: std::string::FromUtf8Error) -> Error
    {
        Error::InvalidUtf8
    }
}

#[cfg(unix)]
impl From<nix::Error> for Error
{
    fn from(error: nix::Error) -> Error
    {
        match error
        {
            nix::Error::InvalidPath => Error::InvalidPath,
            nix::Error::InvalidUtf8 => Error::InvalidUtf8,
            nix::Error::UnsupportedOperation => Error::InvalidPath,
            nix::Error::Sys(errno) => Error::Native("", errno as u32, errno.desc().to_string())
        }
    }
}

impl From<std::ffi::NulError> for Error
{
    fn from(error: std::ffi::NulError) -> Error
    {
        Error::Native("Interior null character found", error.nul_position() as u32, format!("{}", error))
    }
}

