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