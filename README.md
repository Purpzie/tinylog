# tinylog
A logger for my personal projects. Major version increases may occur at any time.

[Documentation](https://docs.rs/tinylog)

## Goals
- **Fast**
  - Write to a thread-local string before writing it to output. This way, all logic can occur before
  the lock, saving time in multi-threaded scenarios.
  - Avoid `dyn` when possible.
  - Avoid `std::fmt::Formatter`. It uses `dyn` and every write produces a `Result`.
- **Minimal**
  - Only provide configuration when it would be difficult to produce the same behavior without it.
  - The default configuration should work for most scenarios.
  - Avoid dependency bloat. Make them optional if possible, and disable their default features.
- **Pretty**
  - Use colors.
  - Print things in a human-friendly format.
