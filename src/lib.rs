/*!
Object implementing `Ensure` trait are in unknown inital state and can be brought to a target state.

By calling `met()` we can be ensured that object is in its target state regardles if it was already in that state or had to be brought to it.
If object was already in target state nothing happens. Otherwise `met()` will call `meet()` on provided `MeetAction` type to bring the object into its target state.

If object implements `Clone` method `met_verify()` can be used to make sure that object fulfills `Met` condition after `MeetAction` type has been preformed.

Closures returning `TryMetResult` that also return closure in `TryMetResult::MeetAction` variant automatically implement `Ensure` trait. 
Helper function `ensure` can be used to call `met()` on such closure.

# Example

This program will create file only if it does not exist.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::TryMetResult::*;

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
```
*/

use std::fmt;
use std::error::Error;

/// Result of verification if object is in target state with `try_met()`
#[derive(Debug)]
pub enum TryMetResult<M, U> {
    Met(M),
    MeetAction(U),
}

#[derive(Debug)]
pub struct UnmetError;

impl fmt::Display for UnmetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "verification of target state failed after it was ensured to be met")
    }
}

impl Error for UnmetError {}

/// Implement for types of objects that can be brought to target state
pub trait Ensure: Sized {
    type Met;
    type MeetAction: MeetAction<Met = Self::Met>;

    /// Check if already `Met` or provide `MeetAction` which can be performed by calling `meet()`
    fn try_met(self) -> TryMetResult<Self::Met, Self::MeetAction>;

    /// Meet the Ensure by calling `try_met()` and if not `Met` calling `meet()` on `MeetAction`
    fn met(self) -> Self::Met {
        match self.try_met() {
            TryMetResult::Met(met) => met,
            TryMetResult::MeetAction(meet) => meet.meet(),
        }
    }

    /// Ensure it is `met()` and then verify it is in fact `Met` with `try_met()`
    fn met_verify(self) -> Result<Self::Met, UnmetError> where Self: Clone {
        let verify = self.clone();
        match self.try_met() {
            TryMetResult::Met(met) => Ok(met),
            TryMetResult::MeetAction(action) => {
                let result = action.meet();
                match verify.try_met() {
                    TryMetResult::Met(_met) => Ok(result),
                    TryMetResult::MeetAction(_action) => Err(UnmetError),
                }
            }
        }
    }
}

/// Function that can be used to bring object in its target state
pub trait MeetAction {
    type Met;

    fn meet(self) -> Self::Met;
}

impl<MET, MA, IMF> Ensure for IMF 
where IMF: FnOnce() -> TryMetResult<MET, MA>, MA: MeetAction<Met = MET> {
    type MeetAction = MA;
    type Met = MET;

    fn try_met(self) -> TryMetResult<Self::Met, Self::MeetAction> {
        self()
    }
}

impl<MET, MF> MeetAction for MF
where MF: FnOnce() -> MET {
    type Met = MET;

    fn meet(self) -> Self::Met {
        self()
    }
}

/// Run `met()` on object implementing `Ensure` and return its value.
/// This is useful with closures implementing `Ensure`.
pub fn ensure<E>(ensure: E) -> E::Met where E:  Ensure {
    ensure.met()
}

/// Mark `T` as something that exists
pub struct Existing<T>(pub T);
/// Mark `T` as something that does not exists
pub struct NonExisting<T>(pub T);

/// Extends any `T` with method allowing to declare it as existing or non-existing
pub trait Existential: Sized {
    fn assume_existing(self) -> Existing<Self> {
        Existing(self)
    }

    fn assume_non_existing(self) -> NonExisting<Self> {
        NonExisting(self)
    }
}

impl<T> Existential for T {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_closure() {
        fn test(met: bool) -> impl Ensure<Met = u8> {
            move || {
                match met {
                    true => TryMetResult::Met(1),
                    _ => TryMetResult::MeetAction(move || 2)
                }
            }
        }

        assert_eq!(test(true).met(), 1);
        assert_eq!(test(false).met(), 2);

        assert_eq!(ensure(test(true)), 1);
        assert_eq!(ensure(test(false)), 2);
    }
}