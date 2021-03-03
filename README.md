# ipipe - A cross-plaform named-pipe library

This library allows the creation of platform-independant named pipes.

Example:
```rust

use ipipe::{Result, Fifo, OnCleanup};
use std::thread;
use std::sync::{Arc, Mutex};

fn main()
{
    let mut fifo = Fifo::create(OnCleanup::Delete)?;
    println!("Fifo path: {}", fifo.path().display());

    let writer = Arc::new(Mutex::from(fifo.clone()));
    let thread = thread::spawn(move || write_nums(&thread_writer));
    print!("{}", fifo.read_string_while(|c| c != CANCEL).unwrap());
}

fn write_nums(fifo: &Mutex<Fifo>) -> Result<usize>
{
    let mut fifo = fifo.lock();
    let fifo = fifo.as_mut().unwrap();
    let mut written = 0;
    for i in 1..=10
    {
        written += fifo.write_string(&format!("{}\n", i))?;
    }
    written += fifo.write_byte(CANCEL)?;
    Ok(written)
}
```
`Fifo::create` generates a random pipe name in a temporary location.
Example path (Windows):
`Fifo path: \\.\pipe\pipe_23676_xMvclVhNKcg6iGf`
Example path (Unix):
`Fifo path: /tmp/pipe_1230_mFP8dx8uVl`

Running the above program will output the same after the fact:
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
