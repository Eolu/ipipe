# Version 0.11.5
- Resolved an [issue](https://github.com/Eolu/ipipe/issues/10).

# Version 0.11.3
- Fixed a permissions issue with Windows pipes. 
- Documented some non-obvious behavior regarding pipe cloning. 

# Version 0.10.0
- Internally to the Pipe struct, raw handles are now wrapped in Arc. Cloning a Pipe results in a weak reference rather than a naive copy of the raw handle. This allows for all clones to be properly invalidated when a handle is closed. Note that this does NOT contain any internal Mutex.
- Minor breaking change - `Pipe::close()` now takes `self` instead of `&mut self`, ensuring that references to closed pipes can't sit around. 

# Version 0.9.0
- Added the `tokio_channels` feature, which provides exactly the same functionality as channels using tokio's async API instead of std threads.

# Version 0.8.2
- No longer delete autocreated pipes on Unix by default. This creates more consistent behavior between unix and Windows.

# Version 0.8
- Added the `channels` feature.

# Version 0.7.5
- Removed a rogue print statement when closing a static pipe.

# Version 0.7.4
- Windows pipes will now attempt to recover from a disconnected client.

# Version 0.7.3
- Fixed a possible deadlock scenario by using a lock-free hashmap.
- Added a close function that can be called explicitly. Pipes are not longer implicitly closed on drop.

# Version 0.7.2
- Unix version will now create pipes if they don't exist when calling `open`. This is to be more consistent with the Windows implementation.

# Version 0.7.1
- Fixed a double flush

# Version 0.7.0
- Finally got reads working correctly for windows and linux pipes. The solution is complex so more testing is stil needed.
- Static pipe print macros now return results. It's better than panicking.

# Version 0.6.1
- Fixed bug that prevented writers from being initialized before readers on Windows. 
- General stability improvements.

# Version 0.6
- Removed dependence on the `windows_named_pipe` crate. Everything is done through winapi on the windows side now. Will likely keep nix on the Linux side as it's sufficiently low-level for any potential purposes of this crate.
- Fixed a bug where compilation failed if the `rand` feature was disabled.

# Version 0.5
- Removed the `Pipe::close()` method. The drop trait now does all the work that once did.
- Renamed some internal interfaces to be more clear.
- Added the `Pipe::name()` method
- `rand` is now optional (default) feature, as its only purpose is a single method that generates a pipe with a random name.
- Documentation updates.

# Version 0.4.2
- Windows paths now use raw strings.
- Documentation cleanup.

# Version 0.4.1
- Made static_pipe a default feature.

# Version 0.4
- Replaced all non-idiomatic I/O interfaces with standard traits (Read, Write, etc). Interfaces should be much more stable now. Static pipes are the exception here, more changes will likely come down the road.
- Slightly better testing and documentation.

# Version 0.3.2
- Fixed a bug with static pipes that prevented their use entirely.

# Version 0.3
- API improvements, better tests, bug fixes, documentation fixes.

# Version 0.2
- Implemented static pipes.

# Version 0.1.1
- Added a Pipe::with_name function that allows for better platform independance.
- Added more documentation.

# Version 0.1
- Initial commit. Lacks features, and likely contains significant bugs. More testing is needed.