use super::{Result, Error, OnCleanup, Handle};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use fcntl::OFlag;
use nix::{fcntl, unistd};
use nix::sys::stat::{stat, Mode, SFlag};
use nix::errno::Errno;
use nix::sys::termios::{tcflush, FlushArg};

#[cfg(feature="rand")]
use rand::{thread_rng, Rng, distributions::Alphanumeric};

/// Abstraction over a named pipe
pub struct Pipe
{
    handle1: Handle,
    handle2: Option<Handle>,
    pub(super) path: PathBuf,
    pub(super) is_slave: bool,
    delete: OnCleanup
}

impl Pipe
{
    /// Open or create a pipe. If on_cleanup is set to 'DeleteOnDrop' the named
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
                        Err(Error::InvalidPath)?;
                    }
                },
                Err(nix::Error::InvalidPath) | Err(nix::Error::Sys(Errno::ENOENT)) => 
                {
                    unistd::mkfifo(path, mode)?;
                },
                err => 
                {
                    err?;
                }
            }

            Pipe::init_handle(path)
                .map(|handle| Pipe 
                { 
                    handle1: handle, 
                    handle2: None,
                    path: path.to_path_buf(), 
                    is_slave: false,
                    delete: on_cleanup
                })
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
        let path = PathBuf::from(format!("/tmp/{}", name));
        Pipe::open(&path, OnCleanup::NoDelete)
    }

    /// Create a pipe with a randomly generated name in a tempory directory.
    #[cfg(feature="rand")]
    pub fn create() -> Result<Self>
    {
        // Generate a random path name
        let path = PathBuf::from(format!("/tmp/pipe_{}_{}", std::process::id(), thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>()));

        Pipe::open(&path, OnCleanup::NoDelete)
    }

    /// Close a named pipe
    pub fn close(self) -> Result<()>
    {
        if let Some(raw) = self.handle1.raw()
        {
            unistd::close(raw).map_err(Error::from)
        }
        else
        {
            Ok(())
        }
    }

    fn init_handle(path: &Path) -> Result<Handle>
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
                .map(|handle| Handle::Arc(Arc::new(handle), HandleType::Unknown))
                .map_err(Error::from)
        }
        else
        {
            Err(Error::InvalidPath)
        }
    }

    fn init_handle_type(&mut self, handle_type: HandleType) -> Result<std::os::unix::io::RawFd>
    {
        if self.handle1.handle_type() == HandleType::Unknown
        {
            self.handle1.set_type(handle_type);
        }
        if self.handle1.handle_type() == handle_type
        {
            self.handle1.raw()
        }
        else
        {
            if let None = self.handle2
            {
                let mut handle = Pipe::init_handle(&self.path)?;
                handle.set_type(handle_type);
                self.handle2 = Some(handle);
            }
            self.handle2.as_ref().unwrap().raw()
        }.ok_or(nix::Error::Sys(nix::errno::Errno::EBADF)).map_err(|e| e.into())
    }
}

impl std::io::Write for Pipe
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> 
    {
        let handle = self.init_handle_type(HandleType::Write)?;
        unistd::write(handle, bytes).map_err(Error::from).map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        let handle = self.init_handle_type(HandleType::Write)?;
        tcflush(handle, FlushArg::TCOFLUSH).map_err(Error::from).map_err(std::io::Error::from)
    }
}

impl std::io::Read for Pipe 
{
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> 
    {
        let handle = self.init_handle_type(HandleType::Read)?;
        unistd::read(handle, bytes)
            .map_err(Error::from)
            .map_err(std::io::Error::from)
    }
}

impl Drop for Pipe
{
    fn drop(&mut self) 
    {
        if !self.is_slave
        {
            self.handle1 = Handle::Weak(Weak::new(), HandleType::Unknown);
            self.handle2 = None;
            if let OnCleanup::Delete = self.delete
            {
                std::fs::remove_file(&self.path).unwrap();
            }
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
            handle1: self.handle1.clone(),
            handle2: self.handle2.clone(),
            path: self.path.clone(), 
            is_slave: true,
            delete: OnCleanup::NoDelete
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum HandleType
{
    Read, Write, Unknown
}
