This library provides `Ensure` trait that is useful for objects with unknown initial state that can be brought to some target state.
For example a file may or may not exist. By implementing `Ensure` we can call `ensure()` to create new file only if it did not exist already.

Closures returning `CheckEnsureResult` that also return closure in `CheckEnsureResult::MeetAction` variant automatically implement `Ensure` trait. 
Helper function `ensure` can be used to call `ensure()` on such closure.

# Example

This program will create file only if it does not exist.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::CheckEnsureResult::*;

fn main() {
    let path = Path::new("/tmp/foo.txt");

    ensure(|| {
        if path.exists() {
            Met(())
        } else {
            MeetAction(|| {
                File::create(&path).unwrap();
            })
        }
    });
}
```
