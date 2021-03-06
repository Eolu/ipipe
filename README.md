# ipipe - A cross-platform named-pipe library for Rust

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

```rust
use ipipe::static_pipe;

static_pipe::init("my_out").unwrap();

let mut reader = static_pipe::reader("my_pipe").unwrap();
println!("String received: {}", reader.read_string_while(|c| c != '\n'));

```
Then anywhere your program (or another program with enough permission to access the pipe) can write code like this:

```rust
pprintln!("my_pipe", "This text will be sent over the pipe!");
```

Lower level & more complete APIs to the static pipes are also planned for a future release. 

# Development Notes

This project is very bare-bones in its current state, a proof-of-concept with some degree of practical usability at best. At this point, developers willing to contribute and improve would be very-much appreciated. Here are some long-term goals for this project:

- High-level interfaces for named pipes that are as simple, easy, and idiomatic to use as Rust's print/println/stdout/stdin. These should be totally platform-agnostic, so it needs to support the lowest-common-denominator list of features at most. This level of interface should trivialize serial IPC!
- Mid-level interfaces for named pipes that give generally platform-agnostic features, but possibly useful platform-specific interfaces as well (which should be separated into specific modules by OS as to ease conditional compilation in cases where these might be used).
- Possibly low-level interfaces, remove reliance on crates like nix and windows_named_pipe.
- Better documentation & testing
- Efficiency improvements. Make it shine.
