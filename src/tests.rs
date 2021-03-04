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