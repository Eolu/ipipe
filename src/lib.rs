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

#[cfg(all(feature="channels", not(feature="tokio_channels")))]
use std::sync::mpsc;

#[cfg(all(feature="tokio_channels", not(feature="channels")))]
use tokio::sync::mpsc;

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

    /// Creates a receiver which all output from this pipe is directed into. A
    /// thread is spawned to read from the pipe, which will shutdown when the 
    /// receiver is dropped. Note that the thread blocks, and may attempt to read
    /// from the pipe one time after the receiver is dropped.
    #[cfg(all(feature="channels", not(feature="tokio_channels")))]
    pub fn receiver(mut self) -> (mpsc::Receiver<u8>, std::thread::JoinHandle<()>)
    {
        use std::io::Read;
        let (tx, rx) = mpsc::channel();
        (rx, 
        std::thread::spawn(move ||
        {
            loop
            {
                for byte in (&mut self).bytes()
                {
                    tx.send(byte.unwrap()).unwrap()
                }
            }
        }))
    }

    /// Creates a receiver which all output from this pipe is directed into. A
    /// task is spawned to read from the pipe, which will shutdown when the 
    /// receiver is dropped. Note that the task blocks, and may attempt to read
    /// from the pipe one time after the receiver is dropped.
    #[cfg(all(feature="tokio_channels", not(feature="channels")))]
    pub async fn receiver(mut self) -> (mpsc::UnboundedReceiver<u8>, tokio::task::JoinHandle<()>)
    {
        use std::io::Read;
        let (tx, rx) = mpsc::unbounded_channel();
        (rx, 
        tokio::task::spawn(async move
        {
            loop
            {
                for byte in (&mut self).bytes()
                {
                    tx.send(byte.unwrap()).unwrap()
                }
            }
        }))
    }

    /// Creates a sender which outputs all input into this pipe. A
    /// thread is spawned to write into the pipe, which will shutdown when the 
    /// sender is dropped.
    #[cfg(all(feature="channels", not(feature="tokio_channels")))]
    pub fn sender(mut self) -> (mpsc::Sender<u8>, std::thread::JoinHandle<()>)
    {
        use std::io::Write;
        let (tx, rx) = mpsc::channel();
        (tx, 
        std::thread::spawn(move ||
        {
            loop
            {
                (&mut self).write(&[rx.recv().unwrap()]).unwrap();
            }
        }))
    }

    /// Creates a sender which outputs all input into this pipe. A
    /// thread is spawned to write into the pipe, which will shutdown when the 
    /// sender is dropped.
    #[cfg(all(feature="tokio_channels", not(feature="channels")))]
    pub fn sender(mut self) -> (mpsc::UnboundedSender<u8>, tokio::task::JoinHandle<()>)
    {
        use std::io::Write;
        let (tx, mut rx) = mpsc::unbounded_channel();
        (tx, 
        tokio::task::spawn(async move
        {
            loop
            {
                (&mut self).write(&[rx.recv().await.unwrap()]).unwrap();
            }
        }))
    }
}

/// Standard error type used by this library
#[derive(Debug)]
pub enum Error
{
    Ipipe(&'static str),
    InvalidPath,
    InvalidUtf8,
    Io(std::io::Error),
    Native(&'static str, u32, String),
    Misc(String)
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self
        {
            Error::Ipipe(s) => s.fmt(f),
            Error::InvalidPath => write!(f, "Invalid path"),
            Error::InvalidUtf8 => write!(f, "Invalid Utf8"),
            Error::Io(err) => err.fmt(f),
            Error::Native(text, code, oss) => write!(f, "{}: {} - {}", text, code, oss),
            Error::Misc(s) => s.fmt(f),
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

impl<'a> From<std::sync::PoisonError<std::sync::MutexGuard<'a, Pipe>>> for Error
{
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<Pipe>>) -> Error
    {
        Error::Misc(err.to_string())
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

