# pyo3-venv
A simple utility library for executing commands within a python venv

I found myself wanting to execute python tests via `cargo test`, thus this crate

## Typical Usage
```rust
#[cfg(test)]
mod tests {
    use pyvenv::PyVEnv;

    #[test]
    fn run_pytest() -> Result<()> {
        PyVenv::new().maturin_develop()?.run_pytest()?
    }
}
```