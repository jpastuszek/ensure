This library provides `Ensure` trait that is useful for objects with unknown initial state that can be brought to some target state.
For example a file may or may not exist. By implementing `Ensure` we can call `met()` to create new file only if it did not exist already.

Closures returning `TryMetResult` that also return closure in `TryMetResult::MeetAction` variant automatically implement `Ensure` trait. 
Helper function `ensure` can be used to call `met()` on such closure.

# Example

This program will create file only if it does not exist.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::TryMetResult::*;

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