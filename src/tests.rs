use crate::Pipe;
use std::thread;
use std::sync::{Arc, Mutex};

const CANCEL: u8 = 24;

#[test]
fn test_pipe() -> crate::Result<()>
{
    let mut pipe = Pipe::create()?;
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

#[test]
fn test_pipe_2() -> crate::Result<()>
{
    let mut pipe = Pipe::create()?;
    println!("Name: {}", pipe.path().display());

    let writer = pipe.clone();
    thread::spawn(move || write_nums_2(writer));
    print!("{}", pipe.read_string_while(|c| c != CANCEL).unwrap());
    Ok(())
}

fn write_nums_2(mut pipe: Pipe) -> crate::Result<usize>
{
    let mut written = 0;
    for i in 1..=10
    {
        written += pipe.write_string(&format!("{}\n", i))?;
    }
    written += pipe.write_byte(CANCEL)?;
    Ok(written)
}

#[cfg(feature="static_pipe")]
#[test]
fn test_static()
{
    const X: char = 'X';
    use crate::static_pipe;

    let mut reader = static_pipe::init("test_pipe").unwrap();

    let thread = thread::spawn(move || reader.read_string_while(|c| c != X as u8));

    thread::sleep(std::time::Duration::from_millis(100));

    pprintln!("test_pipe", "This came through the pipe.");
    pprintln!("test_pipe", "{}", X);
    println!("String sent through the pipe: {:?}", thread.join().unwrap().unwrap());
}
