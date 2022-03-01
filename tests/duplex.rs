use ipipe::Pipe;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;
use std::thread;

use std::io::Write;

fn print_line(buf_reader: &mut BufReader<Pipe>) 
{
    let mut buf = vec![];
    buf_reader.read_until(b'\n', &mut buf).unwrap();
    print!("{}", String::from_utf8(buf).unwrap());
    stdout().flush().unwrap();
}

#[test]
fn duplex_test() 
{
    let mut pipe = Pipe::with_name("test2").unwrap();
    let pipe_clone = pipe.clone();
    let t1 = thread::spawn(||
    {
        writeln!(pipe, "test1").unwrap();
        let mut buf_reader = BufReader::new(pipe);
        print_line(&mut buf_reader);
    });
    let t2 = thread::spawn(|| 
    {
        let mut buf_reader = BufReader::new(pipe_clone);
        print_line(&mut buf_reader);
        let mut pipe_clone = buf_reader.into_inner();
        writeln!(pipe_clone, "test2").unwrap();
    });
    t1.join().unwrap();
    t2.join().unwrap();
}

