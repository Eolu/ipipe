use crate::Pipe;
use lazy_static::lazy_static;
use std::{io::Write, sync::Mutex};
use dashmap::DashMap;

lazy_static! 
{
    static ref PIPES: DashMap<String, Mutex<Pipe>> = DashMap::new();
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
    ($name:tt) => (print_impl($name, "\n"));
    ($name:tt, $($arg:tt)*) => ({ $crate::print($name, format!($($arg)*).as_str()); })
}

/// Initialize a static pipe and return a handle to it.
pub fn init(name: &str) -> crate::Result<Pipe>
{
    let pipe = Pipe::with_name(name)?;
    let reader = pipe.clone();
    PIPES.insert(name.to_string(), Mutex::from(pipe));
    Ok(reader)
}

/// Get a handle to an existing static pipe
pub fn get(name: &str) -> Option<Pipe>
{
    PIPES.get(name).map(|pipe| pipe.lock().unwrap().clone())
}

/// Closes a static pipe
pub fn close(name: &str)
{
    match PIPES.remove(name)
    {
        Some((_, pipe)) => { drop(pipe.lock().unwrap()) }
        None => {}
    }
}

/// Closes all static pipes
pub fn close_all()
{
    PIPES.clear()
}

/// The lowest-level static-pipe print function. Panics if pipe is not 
/// initialized.
#[inline]
pub fn print(name: &str, s: &str)
{
    match PIPES.get(name)
    {
        None => panic!("Pipe not initialized"),
        Some(pipe) => match pipe.lock().as_mut().unwrap().write(s.as_bytes())
        {
            Ok(_) => {}
            Err(e) => panic!(e.to_string())
        }
    }
}
