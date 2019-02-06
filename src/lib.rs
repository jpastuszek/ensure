/*!
Object implementing `Ensure` trait are in unknown inital state and can be brought to a target state.
This can be seen as `Into` trait for side effects on objects in unknown initial state.

By calling `ensure()` we can be ensured that object is in its target state regardles if it was already in that state or had to be brought to it.
If object was already in target state nothing happens. Otherwise `ensure()` will call `meet()` on provided `MeetAction` type to bring the object into its target state.

If object implements `Clone` method `ensure_verify()` can be used to make sure that object fulfills `Met` condition after `MeetAction` type has been preformed.

Closures returning `CheckEnsureResult` that also return closure in `CheckEnsureResult::MeetAction` variant automatically implement `Ensure` trait. 
Helper function `ensure` can be used to call `ensure()` on such closure.

# Example

This program will create file only if it does not exist.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::CheckEnsureResult::*;

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

/// Result of verification if object is in target state with `check_ensure()`
#[derive(Debug)]
pub enum CheckEnsureResult<M, U> {
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

/// Implement for types of objects that can be brought to target state `T`
pub trait Ensure<T>: Sized {
    type MeetAction: MeetAction<Met = T>;

    /// Check if already `Met` or provide `MeetAction` which can be performed by calling `meet()`
    fn check_ensure(self) -> CheckEnsureResult<T, Self::MeetAction>;

    /// Meet the Ensure by calling `check_ensure()` and if not `Met` calling `meet()` on `MeetAction`
    fn ensure(self) -> T {
        match self.check_ensure() {
            CheckEnsureResult::Met(met) => met,
            CheckEnsureResult::MeetAction(meet) => meet.meet(),
        }
    }

    /// Ensure it is `ensure()` and then verify it is in fact `Met` with `check_ensure()`
    fn ensure_verify(self) -> Result<T, UnmetError> where Self: Clone {
        let verify = self.clone();
        match self.check_ensure() {
            CheckEnsureResult::Met(met) => Ok(met),
            CheckEnsureResult::MeetAction(action) => {
                let result = action.meet();
                match verify.check_ensure() {
                    CheckEnsureResult::Met(_met) => Ok(result),
                    CheckEnsureResult::MeetAction(_action) => Err(UnmetError),
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

impl<T, MA, IMF> Ensure<T> for IMF 
where IMF: FnOnce() -> CheckEnsureResult<T, MA>, MA: MeetAction<Met = T> {
    type MeetAction = MA;

    fn check_ensure(self) -> CheckEnsureResult<T, Self::MeetAction> {
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

/// Run `ensure()` on object implementing `Ensure` and return its value.
/// This is useful with closures implementing `Ensure`.
pub fn ensure<E, T>(ensure: E) -> T where E:  Ensure<T> {
    ensure.ensure()
}

/// Mark `T` as something that exists
pub struct Present<T>(pub T);
/// Mark `T` as something that does not exists
pub struct Absent<T>(pub T);

#[cfg(test)]
mod test {
    use super::*;
    use super::CheckEnsureResult;

    #[test]
    fn test_closure() {
        fn test(met: bool) -> impl Ensure<u8> {
            move || {
                match met {
                    true => CheckEnsureResult::Met(1),
                    _ => CheckEnsureResult::MeetAction(move || 2)
                }
            }
        }

        assert_eq!(test(true).ensure(), 1);
        assert_eq!(test(false).ensure(), 2);

        assert_eq!(ensure(test(true)), 1);
        assert_eq!(ensure(test(false)), 2);
    }

    struct Resource;

    struct CreateResourceAction(Resource);
    impl MeetAction for CreateResourceAction {
        type Met = Result<Present<Resource>, ()>;

        fn meet(self) -> Result<Present<Resource>, ()> {
            Ok(Present(self.0))
        }
    }

    impl Ensure<Result<Present<Resource>, ()>> for Resource {
        type MeetAction = CreateResourceAction;

        fn check_ensure(self) -> CheckEnsureResult<Result<Present<Resource>, ()>, Self::MeetAction> {
            if true {
                CheckEnsureResult::Met(Ok(Present(self)))
            } else {
                CheckEnsureResult::MeetAction(CreateResourceAction(self))
            }
        }
    }

    struct DeleteResourceAction(Resource);
    impl MeetAction for DeleteResourceAction {
        type Met = Result<Absent<Resource>, ()>;

        fn meet(self) -> Result<Absent<Resource>, ()> {
            Ok(Absent(self.0))
        }
    }

    impl Ensure<Result<Absent<Resource>, ()>> for Resource {
        type MeetAction = DeleteResourceAction;

        fn check_ensure(self) -> CheckEnsureResult<Result<Absent<Resource>, ()>, Self::MeetAction> {
            if true {
                CheckEnsureResult::Met(Ok(Absent(self)))
            } else {
                CheckEnsureResult::MeetAction(DeleteResourceAction(self))
            }
        }
    }

    #[test]
    fn test_ensure() {
        let _r: Result<Present<Resource>, ()> = Resource.ensure();
        let _r: Result<Absent<Resource>, ()> = Resource.ensure();
    }
}