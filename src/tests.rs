use crate::{Pipe, OnCleanup};
use std::thread;
use std::sync::{Arc, Mutex};

const CANCEL: u8 = 24;

#[test]
fn test_fifo() -> crate::Result<()>
{
    let mut pipe = Pipe::create(OnCleanup::Delete)?;
    println!("Pipe path: {}", pipe.path().display());
    let writer = Arc::new(Mutex::from(pipe.clone()));
    let thread_writer = writer.clone();

    let thread = thread::spawn(move || write_nums(&mut thread_writer.lock().as_mut().unwrap(), 10));

    print!("{}", pipe.read_string_while(|c| c != CANCEL).unwrap());
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    let thread_writer = writer.clone();
    let thread = thread::spawn(move || write_nums(&mut thread_writer.lock().as_mut().unwrap(), 3));

    print!("{}", pipe.read_string_while(|c| c != CANCEL).unwrap());
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    Ok(())
}

#[cfg(feature="static_pipe")]
#[test]
fn test_static()
{
    const X: char = 'X';
    use crate::static_pipe;

    static_pipe::init("test_pipe").unwrap();

    let mut reader = static_pipe::reader("test_pipe").unwrap();
    let thread = thread::spawn(move || reader.read_string_while(|c| c != X as u8));

    thread::sleep(std::time::Duration::from_millis(100));

    pprintln!("test_pipe", "This came through the pipe.");
    pprintln!("test_pipe", "{}", X);
    println!("String sent through the pipe: {:?}", thread.join().unwrap().unwrap());
}

fn write_nums(pipe: &mut Pipe, max: i32) -> crate::Result<usize>
{ 
    let mut written = 0;
    for i in 1..=max
    {
        written += pipe.write_string(&format!("{}\n", i))?;
    }
    written += pipe.write_byte(CANCEL)?;
    Ok(written)
}