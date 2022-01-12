use super::{Result, OnCleanup, Handle};
use std::path::Path;
use std::io::{self, Read, Write};
use std::os::windows::prelude::*;
use std::ffi::OsString;
use std::sync::Arc;
use winapi::
{
    um::winbase::*,
    um::fileapi::*,
    um::handleapi::*,
    um::namedpipeapi::*,
    um::winnt::{GENERIC_READ, GENERIC_WRITE, FILE_ATTRIBUTE_NORMAL},
    shared::winerror::{ERROR_PIPE_NOT_CONNECTED, ERROR_NO_DATA},
    shared::minwindef::{DWORD, LPCVOID, LPVOID}
};

#[cfg(feature="rand")]
use rand::{thread_rng, Rng, distributions::Alphanumeric};

/// Abstraction over a named pipe
#[derive(Debug, Clone)]
pub struct Pipe
{
    read_handle: Option<Handle>,
    write_handle: Option<Handle>,
    pub(super) path: std::path::PathBuf
}

impl Pipe
{
    /// Open a pipe at an existing path. Note that this function is not 
    /// platform-agnostic as unix pipe paths and Windows pipe paths are are 
    /// formatted differently. The second parameter is unused on Windows.
    pub fn open(path: &Path, _: OnCleanup) -> Result<Self>
    {
        Ok(Pipe 
        { 
            read_handle: None,
            write_handle: None,
            path: path.to_path_buf()
        })
    }

    /// Open a pipe with the given name. Note that this is just a string name,
    /// not a path.
    pub fn with_name(name: &str) -> Result<Self>
    {
        let path_string = format!(r"\\.\pipe\{}", name);
        Pipe::open(&Path::new(&path_string), OnCleanup::Delete)
    }

    /// Open a pipe with a randomly generated name.
    #[cfg(feature="rand")]
    pub fn create() -> Result<Self>
    {
        // Generate a random path name
        let path_string = format!(r"\\.\pipe\pipe_{}_{}", std::process::id(),thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .collect::<String>());

        Pipe::open(&Path::new(&path_string), OnCleanup::Delete)
    }

    /// Close a named pipe
    pub fn close(self) -> Result<()>
    {
        if let Some(mut handle) = self.read_handle
        {
            if let Some(raw) = handle.raw()
            {
                unsafe 
                { 
                    if DisconnectNamedPipe(raw) == 0
                    {
                        Err(io::Error::last_os_error())?;
                    }
                }
            }
            handle.set_type(HandleType::Client);
        }
        Ok(())
    }

    /// Creates a new pipe handle
    fn create_pipe(path: &Path) -> io::Result<Handle> 
    {
        let mut os_str: OsString = path.as_os_str().into();
        os_str.push("\x00");
        let u16_slice = os_str.encode_wide().collect::<Vec<u16>>();

        unsafe 
        { 
            while WaitNamedPipeW(u16_slice.as_ptr(), 0xffffffff) == 0 
            {
                let error = io::Error::last_os_error();
                match error.raw_os_error()
                {
                    None => { break; }
                    Some(2) => {}
                    Some(_) => Err(error)?
                }
            } 
        }
        let handle = unsafe 
        {
            CreateFileW(u16_slice.as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        0,
                        std::ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL,
                        std::ptr::null_mut())
        };

        if handle != INVALID_HANDLE_VALUE 
        {
            Ok(Handle::Arc(Arc::new(handle), HandleType::Client))
        } 
        else 
        {
            Err(io::Error::last_os_error())
        }
    }

    /// Creates a pipe listener
    fn create_listener(path: &Path, first: bool) -> io::Result<Handle> 
    {
        let mut os_str: OsString = path.as_os_str().into();
        os_str.push("\x00");
        let u16_slice = os_str.encode_wide().collect::<Vec<u16>>();
        let access_flags = if first 
        {
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE
        } 
        else 
        {
            PIPE_ACCESS_DUPLEX
        };
        let handle = unsafe 
        {
            CreateNamedPipeW(u16_slice.as_ptr(),
                             access_flags,
                             PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                             PIPE_UNLIMITED_INSTANCES,
                             65536,
                             65536,
                             50,
                             std::ptr::null_mut())
        };

        if handle != INVALID_HANDLE_VALUE 
        {
            Ok(Handle::Arc(Arc::new(handle), HandleType::Server))
        } 
        else 
        {
            Err(io::Error::last_os_error())
        }
    }

    /// Initializes the pipe for writing
    fn init_writer(&mut self) -> Result<()>
    {
        if self.write_handle.is_none()
        {
            self.write_handle = Some(Pipe::create_pipe(&self.path)?);
        }
        Ok(())
    }
}

impl std::io::Write for Pipe
{
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> 
    {
        self.init_writer()?;
        let result = match &mut self.write_handle
        {
            None => unreachable!(),
            Some(handle) => handle.write(bytes)
        };

        // Try again if pipe is closed
        match result
        {
            Ok(r) => {return Ok(r);}
            Err(e) if e.raw_os_error().unwrap() as u32 == ERROR_NO_DATA => 
            {
                self.write_handle = None;
                self.init_writer()?;
                match &mut self.write_handle
                {
                    None => unreachable!(),
                    Some(handle) => handle.write(bytes)
                }
            }
            Err(e) => { Err(e)? }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        match &mut self.write_handle
        {
            None => self.init_writer().map_err(std::io::Error::from),
            Some(handle) => 
            {
                handle.flush()?;
                self.write_handle = None;
                Ok(())
            }
        }
    }
}

impl std::io::Read for Pipe
{
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> 
    {
        loop
        {
            let handle = match &mut self.read_handle
            {
                None => 
                {
                    let listener = Pipe::create_listener(&self.path, true)?;
                    // Unwrap is safe because handle was just created
                    if unsafe { ConnectNamedPipe(listener.raw().unwrap(), std::ptr::null_mut()) } == 0 
                    {
                        match io::Error::last_os_error().raw_os_error().map(|x| x as u32) 
                        {
                            Some(ERROR_PIPE_NOT_CONNECTED) => {},
                            Some(err) => Err(io::Error::from_raw_os_error(err as i32))?,
                            _ => unreachable!(),
                        }
                    }
                    self.read_handle = Some(listener);
                    self.read_handle.as_mut().unwrap()
                }
                Some(read_handle) => 
                {
                    if let None = read_handle.raw()
                    {
                        let listener = Pipe::create_listener(&self.path, false)?;
                        // Unwrap is safe because handle was just created
                        if unsafe { ConnectNamedPipe(listener.raw().unwrap(), std::ptr::null_mut()) } == 0 
                        {
                            match io::Error::last_os_error().raw_os_error().map(|x| x as u32) 
                            {
                                Some(ERROR_PIPE_NOT_CONNECTED) => {},
                                Some(err) => Err(io::Error::from_raw_os_error(err as i32))?,
                                _ => unreachable!(),
                            }
                        }
                        self.read_handle = Some(listener);
                        self.read_handle.as_mut().unwrap()
                    }
                    else
                    {
                        read_handle
                    }
                }
            };
    
            match handle.read(bytes)
            {
                Err(e) => 
                {
                    if let Some(err) = e.raw_os_error()
                    {
                        if err as u32 != 109
                        {
                            Err(std::io::Error::from(e))?;
                        }
                        else
                        {
                            continue;
                        }
                    }
                    else
                    {
                        break Ok(0);
                    }
                },
                bytes_read => { break bytes_read; }
            }
        }
    }
}

impl Read for Handle 
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> 
    {
        if let Some(raw) = self.raw()
        {
            let mut bytes_read = 0;
            let ok = unsafe 
            {
                ReadFile(raw,
                         buf.as_mut_ptr() as LPVOID,
                         buf.len() as DWORD,
                         &mut bytes_read,
                         std::ptr::null_mut())
            };
    
            if ok != 0 
            {
                Ok(bytes_read as usize)
            } 
            else 
            {
                match io::Error::last_os_error().raw_os_error().map(|x| x as u32) 
                {
                    Some(ERROR_PIPE_NOT_CONNECTED) => Ok(0),
                    Some(err) => Err(io::Error::from_raw_os_error(err as i32)),
                    _ => unreachable!(),
                }
            }
        }
        else
        {
            Ok(0)
        }
    }
}

impl Write for Handle
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> 
    {
        if let Some(raw) = self.raw()
        {
            let mut bytes_written = 0;
            let status = unsafe 
            {
                WriteFile(raw,
                          buf.as_ptr() as LPCVOID,
                          buf.len() as DWORD,
                          &mut bytes_written,
                          std::ptr::null_mut())
            };
    
            if status != 0 
            {
                Ok(bytes_written as usize)
            } 
            else 
            {
                Err(io::Error::last_os_error())
            }
        }
        else
        {
            Err(io::Error::from_raw_os_error(ERROR_PIPE_NOT_CONNECTED as i32))
        }
    }

    fn flush(&mut self) -> io::Result<()> 
    {
        if let Some(raw) = self.raw()
        {
            if unsafe { FlushFileBuffers(raw) } != 0 
            {
                Ok(())
            } 
            else 
            {
                Err(io::Error::last_os_error())
            }
        }
        else
        {
            Err(io::Error::from_raw_os_error(ERROR_PIPE_NOT_CONNECTED as i32))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum HandleType
{
    Server, Client
}

impl Drop for Handle 
{
    fn drop(&mut self) 
    {
        if let Self::Arc(arc, ty) = self
        {
            let deref = **arc;
            unsafe { FlushFileBuffers(deref); }
            if *ty == HandleType::Server
            {
                unsafe { DisconnectNamedPipe(deref); }
            }
            unsafe { CloseHandle(deref); }
        }
    }
}

