# ipipe - A cross-platorm named-pipe library for Rust

This library allows the creation of platform-independant named pipes. Standard Read/Write traits are implemented. Higher level/more fleshed-out APIs are under development and will be added in future versions. Improvements and PRs welcome.

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

`Pipe::with_name` allows a pipe name to be specified.

# The 'static_pipe' feature
Enabling the `static_pipe` feature allows the creation of mutex-protected static pipes that can be written to from anywhere in a way that mimics stdout. Here's an example:

```
use ipipe::static_pipe;

static_pipe::init("my_out").unwrap();

let mut reader = static_pipe::reader("my_pipe").unwrap();
println!("Byte received: {}", reader.read_string_while(|c| c != '\0'));

```
Then anywhere your program (or another program with enough permission to access the pipe) can write code like this:

```
pprintln!("my_pipe", "This text will be sent over the pipe!{}", '\0');
```

Lower level & more complete APIs to the static pipes are also planned for a future release. 