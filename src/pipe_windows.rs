use super::{Result, Error, OnCleanup};
use std::path::Path;
use windows_named_pipe::{PipeStream, PipeListener};
use std::io::Write;
use rand::{thread_rng, Rng, distributions::Alphanumeric};

/// Abstraction over a named pipe
pub struct Pipe
{
    handle: Option<PipeStream>,
    listener: Option<PipeStream>,
    pub(super) path: std::path::PathBuf,
    pub(super) is_closed: bool
}

unsafe impl Send for Pipe {}
unsafe impl Sync for Pipe {}

impl Pipe
{
    /// Open a pipe at an existing path. Note that this function
    /// is not platform-agnostic as unix pipe paths and Windows 
    /// pipe paths are are formatted differnetly.
    pub fn open(path: &Path, _: OnCleanup) -> Result<Self>
    {
        Ok(Pipe 
        { 
            handle: None,
            listener: None,
            path: path.to_path_buf(), 
            is_closed: false
        })
    }

    /// Open a pipe with the given name. Note that this is just a string name,
    /// not a path.
    pub fn with_name(name: &str) -> Result<Self>
    {
        let path_string = format!(r"\\.\pipe\{}", name);
        Pipe::open(&Path::new(&path_string), OnCleanup::NoDelete)
    }

    /// Open a pipe with a randomly generated name.
    pub fn create() -> Result<Self>
    {
        // Generate a random path name
        let path_string = format!(r"\\.\pipe\pipe_{}_{}", std::process::id(),thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .collect::<String>());

        Pipe::open(&Path::new(&path_string), OnCleanup::NoDelete)
    }

    /// Close the pipe. If the pipe is not closed before deallocation, this will
    /// be called automatically on drop.
    pub fn close(&mut self) -> Result<()>
    {
        self.is_closed = true;
        self.handle = None;
        self.listener = None;
        Ok(())
    }

    /// Flush input and output.
    pub fn flush_pipe(&mut self) -> Result<()>
    {
        // Flush output
        match &mut self.handle
        {
            None => 
            {
                self.init_reader()?;
            }
            Some(_) => 
            {
                self.handle = None;
                self.init_reader()?;
            }
        }

        // Flush input
        match &mut self.listener
        {
            Some(listener) => listener.flush()?,
            None => {}
        }

        Ok(())
    }

    /// Initializes the pipe for reading
    fn init_reader(&mut self) -> Result<()>
    {
        if self.handle.is_none()
        {
            self.handle = Some(PipeStream::connect(&self.path)?);
        }
        Ok(())
    }

    /// Initializes the pipe for writing
    fn init_listener(&mut self) -> Result<()>
    {
        if self.listener.is_none()
        {
            let listener = PipeListener::bind(&self.path).and_then(|mut ls| ls.accept()).map_err(Error::from)?;
            self.listener = Some(listener);
        }
        Ok(())
    }
}

impl std::io::Write for Pipe
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> 
    {
        self.init_reader()?;
        match &mut self.handle
        {
            None => unreachable!(),
            Some(stream) => stream.write(bytes)
        }.map_err(std::io::Error::from)
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        match &mut self.handle
        {
            None => self.init_reader(),
            Some(_) => 
            {
                self.handle = None;
                self.init_reader()
            }
        }.map_err(std::io::Error::from)
    }
}

impl std::io::Read for Pipe
{
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> 
    {
        self.init_listener()?;
        match &mut self.listener
        {
            None => unreachable!(),
            Some(listener) => 
            {
                match listener.read(bytes)
                {
                    Err(e) => 
                    {
                        if let Some(err) = e.raw_os_error()
                        {
                            if err as u32 != 109
                            {
                                Err(std::io::Error::from(e))
                            }
                            else
                            {
                                Ok(0)
                            }
                        }
                        else
                        {
                            Ok(0)
                        }
                    },
                    bytes_read => bytes_read
                }
            }
        }
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
            handle: None,
            listener: None,
            path: self.path.clone(), 
            is_closed: true
        }
    }
}