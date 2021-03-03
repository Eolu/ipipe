# ipipe - A cross-plaform named-pipe library for Rust

This library allows the creation of platform-independant named pipes.

Example:
```rust

use ipipe::{Result, Fifo, OnCleanup};
use std::thread;
use std::sync::{Arc, Mutex};

fn main()
{
    let mut pipe = Pipe::create(OnCleanup::Delete)?;
    println!("Name: {}", pipe.path().display());

    let writer = Arc::new(Mutex::from(pipe.clone()));
    let thread = thread::spawn(move || print_nums(&thread_writer));
    print!("{}", pipe.read_string_while(|c| c != CANCEL).unwrap());
}

fn print_nums(pipe: &Mutex<Fifo>) -> Result<usize>
{
    let mut pipe = pipe.lock();
    let pipe = pipe.as_mut().unwrap();
    let mut written = 0;
    for i in 1..=10
    {
        written += pipe.write_string(&format!("{}\n", i))?;
    }
    written += pipe.write_byte(CANCEL)?;
    Ok(written)
}
```

Running the above example program will output:
```
1
2
3
4
5
6
7
8
9
10
```

`Pipe::create` generates a random pipe name in a temporary location.
Example path (Windows):
`\\.\pipe\pipe_23676_xMvclVhNKcg6iGf`
Example path (Unix):
`/tmp/pipe_1230_mFP8dx8uVl`

