use crate::Pipe;
use lazy_static::lazy_static;
use std::io::Write;
use flurry::*;

// TODO: Accept non-stringly-typed keys somehow
lazy_static! 
{
    static ref PIPES: HashMap<String, Pipe> = HashMap::new();
}

/// Print a string to a static pipe
#[macro_export]
macro_rules! pprint 
{
    ($name:tt, $($arg:tt)*) => ($crate::print($name, format!($($arg)*).as_str()));
}

/// Print a string and a trailing newline to a static pipe
#[macro_export]
macro_rules! pprintln 
{
    ($name:tt) => ($crate::print($name, "\n"));
    ($name:tt, $($arg:tt)*) => ($crate::print($name, {let mut s = format!($($arg)*); s.push('\n'); s}.as_str()))
}

/// Initialize a static pipe and return a handle to it.
pub fn init(name: &str) -> crate::Result<Pipe>
{
    let pipe = Pipe::with_name(name)?;
    let clone = pipe.clone();
    PIPES.insert(name.to_string(), pipe, &PIPES.guard());
    Ok(clone)
}

/// Get a handle to an existing static pipe
pub fn get(name: &str) -> Option<Pipe>
{
    PIPES.get(name, &PIPES.guard()).map(Pipe::clone)
}

/// Closes a static pipe
pub fn close(name: &str)
{
    PIPES.remove(name, &PIPES.guard());
}

/// Closes all static pipes
pub fn close_all()
{
    PIPES.clear(&PIPES.guard())
}

/// The lowest-level static-pipe print function. Panics if pipe is not 
/// initialized.
#[inline]
pub fn print(name: &str, s: &str) -> crate::Result<usize>
{
    match PIPES.get(name, &PIPES.guard()).map(Pipe::clone)
    {
        None => Err(crate::Error::Ipipe("Pipe not initialized")),
        Some(mut pipe) => 
        {
            match pipe.write(s.as_bytes())
            {
                Ok(written) => Ok(written),
                Err(e) => Err(crate::Error::from(e))
            }
        }
    }
}
