# Rim - Rust Text Editor

Rim is a simple text editor written in Rust, inspired by Vim.

## Commands

### Normal Mode
- `i`: Enter Insert mode
- `:`: Enter Command mode
- Arrow keys: Move cursor
- `q`: Quit (in Normal mode only)

### Insert Mode
- Type to insert text
- Arrow keys: Move cursor
- `Backspace`: Delete character before cursor
- `Enter`: Insert new line
- `Esc`: Return to Normal mode

### Command Mode
- `:w`: Save file
- `:q`: Quit
- `:wq`: Save and quit
- `Esc`: Cancel command and return to Normal mode

## Usage

To open a file with Rim:

```
cargo run -- <file_path>
```

If the file doesn't exist, it will be created when you save.
