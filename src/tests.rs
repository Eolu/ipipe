use crate::Pipe;
use std::{io::{Read, Write}, thread};
use std::sync::{Arc, Mutex};

#[test]
fn test_pipe() -> crate::Result<()>
{
    fn write_nums(pipe: &mut Pipe, max: i32) -> crate::Result<usize>
    { 
        let mut written = 0;
        for i in 1..=max
        {
            written += pipe.write(&format!("{}\n", i).as_bytes())?;
        }
        written += pipe.write(&['X' as u8])?;
        Ok(written)
    }
    let mut pipe = Pipe::create()?;
    println!("Pipe path: {}", pipe.path().display());
    let writer = Arc::new(Mutex::from(pipe.clone()));
    let thread_writer = writer.clone();

    let thread = thread::spawn(move || write_nums(&mut thread_writer.lock().as_mut().unwrap(), 10));

    let result = read_until_x(&mut pipe).unwrap();
    print!("{}", result);
    assert_eq!("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n", result);
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    let thread_writer = writer.clone();
    let thread = thread::spawn(move || write_nums(&mut thread_writer.lock().as_mut().unwrap(), 3));

    let result = read_until_x(&mut pipe).unwrap();
    print!("{}", result);
    assert_eq!("1\n2\n3\n", result);
    println!("Bytes sent through the pipe: {:?}", thread.join().unwrap());

    Ok(())
}


#[test]
fn test_pipe_2() -> crate::Result<()>
{
    use std::io::{BufRead, BufWriter};
    let pipe = Pipe::create()?;
    let mut writer = BufWriter::new(pipe.clone());
    thread::spawn(move || -> std::io::Result<()>
        {
            for i in 1..5
            {
                writeln!(&mut writer, "This is line #{}", i)?;
            }
            Ok(())
        });
    for (i, line) in std::io::BufReader::new(pipe).lines().enumerate()
    {
        let line = line?;
        println!("{}", line);
        assert_eq!(format!("This is line #{}", i + 1), line);
        if i == 3
        {
            break;
        }
    }
    Ok(())
}

#[cfg(feature="static_pipe")]
#[test]
fn test_static()
{
    const X: char = 'X';
    use crate::static_pipe;

    let mut reader = static_pipe::init("test_pipe").unwrap();
    let thread = thread::spawn(move || read_until_x(&mut reader));

    thread::sleep(std::time::Duration::from_millis(100));

    pprintln!("test_pipe", "This came through the pipe.");
    pprintln!("test_pipe", "{}", X);
    let result = thread.join().unwrap().unwrap();
    println!("String sent through the pipe: {:?}", result);
    assert_eq!("This came through the pipe.", result);
}

fn read_until_x(pipe: &mut Pipe) -> std::io::Result<String>
{
    let mut buf: [u8; 1] = [0];
    let mut container = String::new();
    loop
    {
        match pipe.read(&mut buf)
        {
            Ok(_) if buf[0] != 'X' as u8 => container.push(buf[0] as char),
            _ => { break Ok(container); }
        }
    }
}