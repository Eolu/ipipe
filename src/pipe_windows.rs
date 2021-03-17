use super::{Result, OnCleanup};
use std::path::Path;
use std::io::{self, Read, Write};
use std::os::windows::prelude::*;
use std::ffi::OsString;
use winapi::
{
    um::winbase::*,
    um::fileapi::*,
    um::handleapi::*,
    um::namedpipeapi::*,
    um::winnt::{HANDLE, GENERIC_READ, GENERIC_WRITE, FILE_ATTRIBUTE_NORMAL},
    shared::winerror::ERROR_PIPE_NOT_CONNECTED,
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

unsafe impl Send for Pipe {}
unsafe impl Sync for Pipe {}

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
            Ok(Handle { inner: handle, handle_type: HandleType::Client})
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
            FILE_FLAG_FIRST_PIPE_INSTANCE
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
            Ok(Handle { inner: handle, handle_type: HandleType::Server })
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
        match &mut self.write_handle
        {
            None => unreachable!(),
            Some(handle) => handle.write(bytes)
        }.map_err(std::io::Error::from)
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
        if let None = self.read_handle
        {
            let listener = Pipe::create_listener(&self.path, true)?;
            if unsafe { ConnectNamedPipe(listener.inner, std::ptr::null_mut()) } == 0 
            {
                match io::Error::last_os_error().raw_os_error().map(|x| x as u32) 
                {
                    Some(ERROR_PIPE_NOT_CONNECTED) => {},
                    Some(err) => Err(io::Error::from_raw_os_error(err as i32))?,
                    _ => unreachable!(),
                }
            }
            self.read_handle = Some(listener)
        }
        match &mut self.read_handle
        {
            None => unreachable!(),
            Some(read_handle) => 
            {
                match read_handle.read(bytes)
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

impl Read for Handle 
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> 
    {
        let mut bytes_read = 0;
        let ok = unsafe 
        {
            ReadFile(self.inner,
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
            match io::Error::last_os_error().raw_os_error().map(|x| x as u32) {
                Some(ERROR_PIPE_NOT_CONNECTED) => Ok(0),
                Some(err) => Err(io::Error::from_raw_os_error(err as i32)),
                _ => panic!(""),
            }
        }
    }
}

impl Write for Handle
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> 
    {
        let mut bytes_written = 0;
        let status = unsafe 
        {
            WriteFile(self.inner,
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

    fn flush(&mut self) -> io::Result<()> 
    {
        if unsafe { FlushFileBuffers(self.inner) } != 0 
        {
            Ok(())
        } 
        else 
        {
            Err(io::Error::last_os_error())
        }
    }
}

#[derive(Debug)]
enum HandleType
{
    Server, Slave, Client
}

#[derive(Debug)]
struct Handle 
{
    inner: HANDLE,
    handle_type: HandleType
}

impl Clone for Handle
{
    fn clone(&self) -> Self 
    {
        Handle
        {
            inner: self.inner,
            handle_type: HandleType::Slave
        }
    }
}

impl Drop for Handle 
{
    fn drop(&mut self) 
    {
        unsafe { FlushFileBuffers(self.inner); }
        match self.handle_type
        {
            HandleType::Slave => { return; }
            HandleType::Server => unsafe { DisconnectNamedPipe(self.inner); }
            HandleType::Client => {}
        }
        unsafe { CloseHandle(self.inner); }
    }
}

unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}