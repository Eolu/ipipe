use super::{Result, Error, OnCleanup};
use std::{io::Read, path::Path};
use windows_named_pipe::{PipeStream, PipeListener};
use std::io::Write;
use rand::{thread_rng, Rng, distributions::Alphanumeric};

/// Abstraction over a named pipe
pub struct Fifo
{
    handle: Option<PipeStream>,
    listener: Option<PipeStream>,
    pub(super) path: std::path::PathBuf,
    pub(super) is_closed: bool
}

unsafe impl Send for Fifo {}
unsafe impl Sync for Fifo {}

impl Fifo
{
    /// Open an existing pipe. If 'delete_on_drop' is true, the named pipe will
    /// be deleted when the returned struct is deallocated.
    pub fn open(path: &Path, _: OnCleanup) -> Result<Self>
    {
        Ok(Fifo 
        { 
            handle: None,
            listener: None,
            path: path.to_path_buf(), 
            is_closed: false
        })
    }

    /// Create a pipe. If 'delete_on_drop' is true, the named pipe will be
    /// deleted when the returned struct is deallocated.
    pub fn create(delete_on_drop: OnCleanup) -> Result<Self>
    {
        // Generate a random path name
        let path_string = format!("\\\\.\\pipe\\pipe_{}_{}", std::process::id(),thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .collect::<String>());

        Fifo::open(&Path::new(&path_string), delete_on_drop)
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

    /// Write a single byte to the pipe
    pub fn write_byte(&mut self, buf: u8) -> Result<usize>
    {
        self.write_bytes(&[buf])
    }

    /// Write an array of bytes to the pipe
    pub fn write_bytes(&mut self, buf: &[u8]) -> Result<usize>
    {
        self.init_reader()?;
        match &mut self.handle
        {
            None => unreachable!(),
            Some(stream) => stream.write(buf)
        }.map_err(Error::from)
    }

    /// Writes a string to the pipe
    pub fn write_string(&mut self, s: &str) -> Result<usize>
    {
        self.init_reader()?;
        self.write_bytes(s.as_bytes())
    }

    /// Read a single byte
    pub fn read_byte(&mut self) -> Result<u8>
    {
        self.init_listener()?;
        match &mut self.listener
        {
            None => unreachable!(),
            Some(listener) => 
            {
                let buf = &mut [0 as u8];
                match listener.read(buf)
                {
                    Err(e) => 
                    {
                        if let Some(err) = e.raw_os_error()
                        {
                            if err as u32 != 109
                            {
                                return Err(Error::from(e));
                            }
                        }
                    },
                    _ => ()
                }
                Ok(buf[0])
            }
        }
    }

    /// Reads the given number of bytes and returns the result in a vector.
    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>>
    {
        self.init_listener()?;
        match &mut self.listener
        {
            None => unreachable!(),
            Some(listener) => 
            {
                let mut buf = Vec::with_capacity(size);
                match listener.read_exact(&mut buf)
                {
                    Err(e) => 
                    {
                        if let Some(err) = e.raw_os_error()
                        {
                            if err as u32 != 109
                            {
                                return Err(Error::from(e));
                            }
                        }
                    },
                    _ => ()
                }
                Ok(buf)
            }
        }
    }

    /// Reads the given number of bytes and returns the result as a string.
    pub fn read_string(&mut self, size: usize) -> Result<String>
    {
        self.read_bytes(size).map(|buf| String::from_utf8_lossy(&buf).into_owned())
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

    /// Initializes the FIFO for reading
    fn init_reader(&mut self) -> Result<()>
    {
        if self.handle.is_none()
        {
            self.handle = Some(PipeStream::connect(&self.path)?);
        }
        Ok(())
    }

    /// Initializes the FIFO for writing
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

impl std::io::Read for Fifo
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

impl Drop for Fifo
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

impl Clone for Fifo
{
    /// Cloning a fifo creates a slave which points to the same path but does not
    /// close the fifo when dropped.
    fn clone(&self) -> Self 
    {
        Fifo 
        { 
            handle: None,
            listener: None,
            path: self.path.clone(), 
            is_closed: true
        }
    }
}