[![Latest Version]][crates.io] [![Documentation]][docs.rs] ![License]

This library provides `Ensure` trait that is useful for objects with unknown initial external state that can be brought to some target state.

This can be seen as `TryInto` trait for objects with side effects with unknown initial external state and desired target state.
For example a file may or may not exist. By implementing `Ensure` we can call `ensure()` to create new file only if it did not exist already.

Closures returning `CheckEnsureResult` that also return closure in `CheckEnsureResult::EnsureAction` variant automatically implement `Ensure` trait.
Helper function `ensure` can be used to call `ensure()` on such closure.

# Example

This program will create file only if it does not exist already.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::CheckEnsureResult::*;

fn main() {
    let path = Path::new("/tmp/foo.txt");

    ensure(|| {
        Ok(if path.exists() {
            Met(())
        } else {
            EnsureAction(|| {
                File::create(&path).map(|file| drop(file))
            })
        })
    }).expect("failed to create file");
}
```

[crates.io]: https://crates.io/crates/ensure
[Latest Version]: https://img.shields.io/crates/v/ensure.svg
[Documentation]: https://docs.rs/ensure/badge.svg
[docs.rs]: https://docs.rs/ensure
[License]: https://img.shields.io/crates/l/ensure.svg
