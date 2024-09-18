# NES emulator in Rust

## Workflow

All these commands should be run from the root folder and should assume the Rust toolchain presence on the computer.

### Running the emulator

```bash
cargo run -- <rom_file>
```

### Running clippy on the code

Run `cargo clippy --fix --allow-dirty` to fix clippy issues in the code.
