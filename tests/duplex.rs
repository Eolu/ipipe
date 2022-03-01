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

fn client_server1(mut pipe: Pipe) 
{
    writeln!(pipe, "test1").unwrap();
    writeln!(pipe, "test2").unwrap();
    writeln!(pipe, "test3").unwrap();
    let mut buf_reader = BufReader::new(pipe);
    print_line(&mut buf_reader);
    print_line(&mut buf_reader);
    print_line(&mut buf_reader);
}

fn client_server2(pipe: Pipe) 
{
    let mut buf_reader = BufReader::new(pipe);
    print_line(&mut buf_reader);
    print_line(&mut buf_reader);
    print_line(&mut buf_reader);
    let mut pipe = buf_reader.into_inner();
    writeln!(pipe, "test4").unwrap();
    writeln!(pipe, "test5").unwrap();
    writeln!(pipe, "test6").unwrap();
}

#[test]
fn duplex_test() 
{
    let pipe = Pipe::with_name("test2").unwrap();
    let pipe_clone = pipe.clone();
    let t1 = thread::spawn(|| client_server1(pipe));
    let t2 = thread::spawn(|| client_server2(pipe_clone));
    t1.join().unwrap();
    t2.join().unwrap();
}
