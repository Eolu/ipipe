use super::{Result, Error, OnCleanup};
use std::path::{Path, PathBuf};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use fcntl::OFlag;
use nix::{fcntl, unistd};
use nix::sys::stat::{stat, Mode, SFlag};
use nix::errno::Errno;
use nix::sys::termios::{tcflush, FlushArg};

/// Abstraction over a named pipe
pub struct Pipe
{
    handle: std::os::unix::io::RawFd,
    pub(super) path: PathBuf,
    pub(super) is_closed: bool,
    delete: OnCleanup
}

impl Pipe
{
    /// Open an existing pipe. If on_cleanup is set to 'DeleteOnDrop' the named
    /// pipe will be deleted when the returned struct is deallocated.
    /// Note that this function is not platform-agnostic as unix pipe paths and 
    /// Windows pipe paths are formatted differnetly.
    pub fn open(path: &Path, on_cleanup: OnCleanup) -> Result<Self>
    {
        let mode = Mode::S_IWUSR | Mode::S_IRUSR 
                 | Mode::S_IRGRP | Mode::S_IWGRP;

        if let Some(_) = path.parent()
        {
            match stat(path)
            {
                Ok(file_stat) => 
                {
                    // Error out if file is not a named pipe
                    if file_stat.st_mode & SFlag::S_IFIFO.bits() == 0
                    {
                        Err(nix::Error::InvalidPath)?;
                    }
                },
                err => 
                {
                    err?;
                }
            }

            fcntl::open(path, OFlag::O_RDWR | OFlag::O_NOCTTY, mode)
                .map(|handle| Pipe 
                    { 
                        handle, 
                        path: path.to_path_buf(), 
                        is_closed: false,
                        delete: on_cleanup
                    }).map_err(Error::from)
        }
        else
        {
            Err(Error::InvalidPath)
        }
    }

    /// Open or create a pipe with the given name. Note that this is just a
    /// string name, not a path.
    pub fn with_name(name: &str) -> Result<Self>
    {
        let mode = Mode::S_IWUSR | Mode::S_IRUSR 
                 | Mode::S_IRGRP | Mode::S_IWGRP;
        
        let path = PathBuf::from(format!("/tmp/{}", name));
        if let Some(_) = path.parent()
        {
            match stat(&path)
            {
                Ok(file_stat) => 
                {
                    // Error out if file is not a named pipe
                    if file_stat.st_mode & SFlag::S_IFIFO.bits() == 0
                    {
                        Err(Error::InvalidPath)?;
                    }
                },
                Err(nix::Error::InvalidPath) | Err(nix::Error::Sys(Errno::ENOENT)) => 
                {
                    unistd::mkfifo(&path, mode)?;
                },
                err => 
                {
                    err?;
                }
            }

            Pipe::open(&path, OnCleanup::NoDelete)
        }
        else
        {
            Err(Error::InvalidPath)
        }
    }

    /// Create a pipe with a randomly generated name in a tempory directory. If
    /// on_cleanup is set to 'DeleteOnDrop' the named pipe will be deleted when
    /// the returned struct is deallocated.
    pub fn create() -> Result<Self>
    {
        let mode = Mode::S_IWUSR | Mode::S_IRUSR 
                 | Mode::S_IRGRP | Mode::S_IWGRP;

        // Generate a random path name
        let path = PathBuf::from(format!("/tmp/pipe_{}_{}", std::process::id(), thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>()));

        if let Some(_) = path.parent()
        {
            match stat(&path)
            {
                Ok(file_stat) => 
                {
                    // Error out if file is not a named pipe
                    if file_stat.st_mode & SFlag::S_IFIFO.bits() == 0
                    {
                        Err(Error::InvalidPath)?;
                    }
                },
                Err(nix::Error::InvalidPath) | Err(nix::Error::Sys(Errno::ENOENT)) => 
                {
                    unistd::mkfifo(&path, mode)?;
                },
                err => 
                {
                    err?;
                }
            }

            Pipe::open(&path, OnCleanup::NoDelete)
        }
        else
        {
            Err(Error::InvalidPath)
        }
    }

    /// Close the pipe. If the pipe is not closed before deallocation, this will
    /// be called automatically on drop.
    pub fn close(&mut self) -> Result<()>
    {
        self.is_closed = true;
        unistd::close(self.handle).map_err(Error::from)
    }

    /// Flush input and output.
    pub fn flush_pipe(&self) -> Result<()>
    {
        tcflush(self.handle, FlushArg::TCIOFLUSH).map_err(Error::from)
    }
}

impl std::io::Write for Pipe
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> 
    {
        unistd::write(self.handle, bytes).map_err(Error::from).map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        tcflush(self.handle, FlushArg::TCOFLUSH).map_err(Error::from).map_err(std::io::Error::from)
    }
}

impl std::io::Read for Pipe 
{
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> 
    {
        unistd::read(self.handle, bytes)
            .map_err(Error::from)
            .map_err(std::io::Error::from)
    }
}

impl Drop for Pipe
{
    fn drop(&mut self) 
    {
        if !self.is_closed
        {
            if let Err(e) = self.close()
            {
                eprintln!("Error closing pipe: {:?}", e)
            }
        }

        if let OnCleanup::Delete = self.delete
        {
            std::fs::remove_file(&self.path).expect("Pipe removal Failed")
        }
    }
}

impl Clone for Pipe
{
    /// Cloning a pipe creates a slave which points to the same path but does not
    /// close the pipe when dropped.
    fn clone(&self) -> Self 
    {
        Pipe 
        { 
            handle: self.handle,
            path: self.path.clone(), 
            is_closed: true,
            delete: OnCleanup::NoDelete
        }
    }
}
