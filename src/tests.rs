use crate::{Result, Fifo, OnCleanup};
use std::thread;
use std::sync::{Arc, Mutex};

const CANCEL: u8 = 24;

#[test]
fn test_fifo() -> Result<()>
{
    let mut fifo = Fifo::create(OnCleanup::Delete)?;
    println!("Fifo path: {}", fifo.path().display());
    let writer = Arc::new(Mutex::from(fifo.clone()));
    let thread_writer = writer.clone();

    let thread = thread::spawn(move || write_nums(&thread_writer, 10));

    print!("{}", fifo.read_string_while(|c| c != CANCEL).unwrap());
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    let thread_writer = writer.clone();
    let thread = thread::spawn(move || write_nums(&thread_writer, 3));

    print!("{}", fifo.read_string_while(|c| c != CANCEL).unwrap());
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    Ok(())
}

fn write_nums(fifo: &Mutex<Fifo>, max: i32) -> Result<usize>
{
    let mut fifo = fifo.lock();
    let fifo = fifo.as_mut().unwrap();
    let mut written = 0;
    for i in 1..=max
    {
        written += fifo.write_string(&format!("{}\n", i))?;
    }
    written += fifo.write_byte(CANCEL)?;
    Ok(written)
}