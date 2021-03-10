//! Cross-platform named-pipe API.
//!
//! # Quick Start
//!
//! To get started quickly, try using Pipe::with_name to create a pipe with a 
//! given name.
//! ```
//! use ipipe::Pipe; 
//! use std::io::BufRead;
//! fn reader()
//! {
//!     let mut pipe = Pipe::with_name("test_pipe").unwrap();
//!     println!("Pipe path: {}", pipe.path().display());
//!
//!     // Read lines
//!     for line in std::io::BufReader::new(pipe).lines()
//!     {
//!         println!("{}", line.unwrap());
//!     }
//! }
//! ```
//! 
//! Then in another program or thread:
//! ```
//! use ipipe::Pipe; 
//! use std::io::Write;
//! fn writer()
//! {
//!     let mut pipe = Pipe::with_name("test_pipe").unwrap();
//!     writeln!(&mut pipe, "This is only a test.").unwrap();
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
mod pipe_unix;
#[cfg(unix)]
pub use pipe_unix::*;

#[cfg(windows)]
mod pipe_windows;
#[cfg(windows)]
pub use pipe_windows::*;

#[cfg(feature="static_pipe")]
#[macro_use]
mod static_pipe;
#[cfg(feature="static_pipe")]
pub use static_pipe::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub enum OnCleanup
{
    Delete,
    NoDelete
}

impl Pipe
{
    /// Return the path to this named pipe
    pub fn path(&self) -> &std::path::Path
    {
        &self.path
    }

    /// Gets the name of this pipe
    pub fn name(&self) -> Option<&std::ffi::OsStr>
    {
        self.path().file_name()
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

