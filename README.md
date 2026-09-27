# tinylog
![Release version](https://img.shields.io/github/v/release/Purpzie/tinylog)
![No AI](https://img.shields.io/badge/No_AI-green.svg?logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNzYiIGhlaWdodD0iMjc2Ij48Y2lyY2xlIGN4PSIxMzgiIGN5PSIxMzgiIHI9IjEyMCIgZmlsbD0iIzAwMCIvPjxjaXJjbGUgY3g9IjEzOCIgY3k9IjEzOCIgcj0iMTI0IiBzdHJva2U9IiNCMzAwMDAiIHN0cm9rZS13aWR0aD0iMjgiIGZpbGw9Im5vbmUiLz48cmVjdCB4PSI4MCIgeT0iMTQiIHdpZHRoPSIzMiIgaGVpZ2h0PSIyNDIiIGZpbGw9IiNGRkYiIHRyYW5zZm9ybT0ic2tld1goLTkpIi8+PHJlY3QgeD0iOTIiIHk9IjE0IiB3aWR0aD0iMzIiIGhlaWdodD0iMjQyIiBmaWxsPSIjRkZGIiB0cmFuc2Zvcm09InNrZXdYKDkpIi8+PHJlY3QgeD0iNzgiIHk9IjE3MyIgd2lkdGg9IjUwIiBoZWlnaHQ9IjMyIiBmaWxsPSIjRkZGIi8+PHJlY3QgeD0iMTgwIiB5PSIxNSIgd2lkdGg9IjM2IiBoZWlnaHQ9IjIzMCIgZmlsbD0iI0ZGRiIvPjxjaXJjbGUgY3g9IjEzOCIgY3k9IjEzOCIgcj0iMTI0IiBzdHJva2U9IiNCMzAwMDAiIHN0cm9rZS13aWR0aD0iMjgiIGZpbGw9Im5vbmUiIHN0cm9rZS1kYXNoYXJyYXk9IjM5MCIvPjxsaW5lIHgxPSI0NSIgeTE9IjQ1IiB4Mj0iMjMxIiB5Mj0iMjMxIiBzdHJva2U9IiNCMzAwMDAiIHN0cm9rZS13aWR0aD0iMjIiLz48L3N2Zz4=)

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
