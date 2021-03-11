# ipipe - A cross-platform named-pipe library for Rust

This library allows the creation of platform-independant named pipes. Standard Read/Write traits are implemented. APIs and performance will be improved in future versions. Issues and PRs welcome.

Example:
```rust

use ipipe::Pipe;
use std::thread;
use std::io::{BufRead, BufWriter};

const CANCEL: u8 = 24;

fn main()
{
    let mut pipe = Pipe::create().unwrap();
    println!("Name: {}", pipe.path().display());

    let writer = pipe.clone();
    thread::spawn(move || print_nums(writer));
    for line in BufReader::new(pipe).lines()
    {
        println!("{}", line.unwrap());
    }
}

fn print_nums(mut pipe: Pipe)
{
    for i in 1..=10
    {
        writeln!(&mut pipe, "{}", i).unwrap();
    }
    write!(&mut pipe, "{}", CANCEL as char).unwrap();
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


# Features
- ### static_pipe
The `static_pipe` default feature allows the creation of mutex-protected static pipes that can be written to from anywhere in a way that mimics stdout. Here's an example:

```rust
use ipipe::*;
use std::io::{BufRead, BufWriter};

let mut reader = ipipe::init("my_out").unwrap();

// You can get a handle to an already-initialized pipe like this:
// let mut reader = ipipe::get("my_pipe");
let s = BufReader::new(pipe).lines().next().unwrap();
println!("String received: {}", s);

// Drops the static pipe. Can also call `ipipe::close_all()` to drop all static pipes.
ipipe::close("my_out");
```
Then anywhere your program (or another program with enough permission to access the pipe) can write code like this:

```rust
pprintln!("my_pipe", "This text will be sent over the pipe!");
```

Lower level as well as more complete/intuitive APIs to the static pipes are also planned for a future release. 

- ### rand
The 'rand' default feature will allow calling `Pipe::create()` to open a pipe with a randomly-generated name. The generated name will have the following format: `pipe_[process pid]_[15 random alphnumeric characters]`. Equivalent to `Pipe::with_name(&str)` in every other way.
