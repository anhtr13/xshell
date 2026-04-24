# Xshell

A Unix-like shell implementation in Rust, built for the [Build Your Own Shell](https://app.codecrafters.io/courses/shell/overview) challenge on [Codecrafters](https://app.codecrafters.io).

## Build & Run

```bash
# Release build
cargo build --release

# Run the shell
./target/release/xshell
```

Or use the project script (builds then runs):

```bash
./your_program.sh
```

## Features

| Feature             | Description                                                                                                             |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| **Commands**        | Built-ins (`cd`, `echo`, `exit`, `history`, `pwd`, `type`, `jobs`) and external programs from `PATH`                    |
| **Editing**         | Arrow keys (←/→), backspace, insert at cursor; full line redraw keeps display in sync                                   |
| **History**         | Persistent history via `HISTFILE`; ↑/↓ to navigate; `history` built-in with `-c`, `-r`, `-w`, `-a`                      |
| **Completion**      | Tab completion for executables and paths                                                                                |
| **Pipelines**       | Chain commands with `\|`                                                                                                |
| **Redirection**     | `>`, `>>`, `2>`, `2>>`, `&>`, `&>>` for stdout/stderr                                                                   |
| **Background jobs** | Run command in background with `&` ending; builtin `jobs` list all current jobs; clean finished jobs before each prompt |
