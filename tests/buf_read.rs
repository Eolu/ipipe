use ipipe::Pipe;
use std::io::{BufRead, Write, Read};
use std::thread;
use std::process::Command;

fn reader(mut pipe: Pipe) 
{
    let mut buffer = String::new();
    //let mut buffer2 = vec!('x' as u8, 'x' as u8, 'x' as u8, 'x' as u8, 'x' as u8, 'x' as u8, 'x' as u8, 'x' as u8);
    let mut reader = std::io::BufReader::new(&mut pipe);
    loop 
    {
        // match pipe.read(&mut buffer2)
        // {
        //     Ok(bytes_read) => 
        //     {
        //         println!("read bytes: {:?}", bytes_read);
        //         if bytes_read == 0 { break }
        //         println!("{:?}", String::from_utf8_lossy(&buffer2));
    
        //     }
        //     Err(err) => 
        //     {
        //         println!("{:?}", err);
        //         //return Err(err);
        //     }
        // }
        match reader.read_line(&mut buffer) 
        {
            Ok(bytes_read) => 
            {
                println!("read bytes: {:?}", bytes_read);
                if bytes_read == 0 { break }
                println!("Buffer: {:?}", buffer);
    
            }
            Err(err) => 
            {
                println!("{:?}", err);
                //return Err(err);
            }
        };
    }
}

fn writer(mut pipe: Pipe) 
{
    let output = if cfg!(target_os = "windows") 
    {
        Command::new("cmd")
                .args(["/C", "cat tests/test_in"])
                .output()
                .expect("failed to read test_in")
    } 
    else 
    {
        Command::new("sh")
                .arg("-c")
                .arg("cat tests/test_in")
                .output()
                .expect("failed to read test_in")
    };
    //let output = output.stdout;
    let read = std::fs::read("tests/test_in").unwrap();
    pipe.write_all(&read).unwrap();
    pipe.flush().unwrap();
    drop(pipe)
}

#[test]
fn buf_read_test() 
{
    let t1 = thread::spawn(|| reader(Pipe::with_name("buf_read_test").unwrap()));
    let t2 = thread::spawn(|| writer(Pipe::with_name("buf_read_test").unwrap()));
    t1.join().unwrap();
    t2.join().unwrap();
}
