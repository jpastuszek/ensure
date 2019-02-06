/*!
Object implementing `Ensure` trait are in unknown inital state and can be brought to a target state.
This can be seen as `Into` trait for side effects on objects in unknown initial state.

By calling `ensure()` we can be ensured that object is in its target state regardles if it was already in that state or had to be brought to it.
If object was already in target state nothing happens. Otherwise `ensure()` will call `meet()` on provided `EnsureAction` type to bring the object into its target state.

If object implements `Clone` method `ensure_verify()` can be used to make sure that object fulfills `Met` condition after `EnsureAction` type has been preformed.

Closures returning `CheckEnsureResult` that also return closure in `CheckEnsureResult::EnsureAction` variant automatically implement `Ensure` trait. 
Helper function `ensure` can be used to call `ensure()` on such closure.

# Example

This program will create file only if it does not exist already.

```rust
use std::path::Path;
use std::fs::File;
use ensure::ensure;
use ensure::CheckEnsureResult::*;

let path = Path::new("/tmp/foo.txt");

ensure(|| {
    if path.exists() {
        Met(File::open(&path))
    } else {
        EnsureAction(|| {
            File::create(&path)
        })
    }
}).expect("failed to open file");
```
*/

use std::fmt;
use std::error::Error;

/// Result of verification if object is in target state with `check_ensure()`
#[derive(Debug)]
pub enum CheckEnsureResult<M, A> {
    Met(M),
    EnsureAction(A),
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
    type EnsureAction: Meet<Met = T>;

    /// Check if already `Met` or provide `EnsureAction` which can be performed by calling `meet()`
    fn check_ensure(self) -> CheckEnsureResult<T, Self::EnsureAction>;

    /// Meet the Ensure by calling `check_ensure()` and if not `Met` calling `meet()` on `EnsureAction`
    fn ensure(self) -> T {
        match self.check_ensure() {
            CheckEnsureResult::Met(met) => met,
            CheckEnsureResult::EnsureAction(meet) => meet.meet(),
        }
    }

    /// Ensure it is `ensure()` and then verify it is in fact `Met` with `check_ensure()`
    fn ensure_verify(self) -> Result<T, UnmetError> where Self: Clone {
        let verify = self.clone();
        match self.check_ensure() {
            CheckEnsureResult::Met(met) => Ok(met),
            CheckEnsureResult::EnsureAction(action) => {
                let result = action.meet();
                match verify.check_ensure() {
                    CheckEnsureResult::Met(_met) => Ok(result),
                    CheckEnsureResult::EnsureAction(_action) => Err(UnmetError),
                }
            }
        }
    }
}

/// Function that can be used to bring object in its target state
pub trait Meet {
    type Met;

    fn meet(self) -> Self::Met;
}

impl<T, A, F> Ensure<T> for F 
where F: FnOnce() -> CheckEnsureResult<T, A>, A: Meet<Met = T> {
    type EnsureAction = A;

    fn check_ensure(self) -> CheckEnsureResult<T, Self::EnsureAction> {
        self()
    }
}

impl<T, F> Meet for F
where F: FnOnce() -> T {
    type Met = T;

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
    use super::CheckEnsureResult::*;

    #[test]
    fn test_closure() {
        fn test(met: bool) -> impl Ensure<u8> {
            move || {
                match met {
                    true => Met(1),
                    _ => EnsureAction(move || 2)
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
    impl Meet for CreateResourceAction {
        type Met = Result<Present<Resource>, ()>;

        fn meet(self) -> Result<Present<Resource>, ()> {
            Ok(Present(self.0))
        }
    }

    impl Ensure<Result<Present<Resource>, ()>> for Resource {
        type EnsureAction = CreateResourceAction;

        fn check_ensure(self) -> CheckEnsureResult<Result<Present<Resource>, ()>, Self::EnsureAction> {
            if true {
                Met(Ok(Present(self)))
            } else {
                EnsureAction(CreateResourceAction(self))
            }
        }
    }

    struct DeleteResourceAction(Resource);
    impl Meet for DeleteResourceAction {
        type Met = Result<Absent<Resource>, ()>;

        fn meet(self) -> Result<Absent<Resource>, ()> {
            Ok(Absent(self.0))
        }
    }

    impl Ensure<Result<Absent<Resource>, ()>> for Resource {
        type EnsureAction = DeleteResourceAction;

        fn check_ensure(self) -> CheckEnsureResult<Result<Absent<Resource>, ()>, Self::EnsureAction> {
            if true {
                Met(Ok(Absent(self)))
            } else {
                EnsureAction(DeleteResourceAction(self))
            }
        }
    }

    #[test]
    fn test_ensure() {
        let _r: Result<Present<Resource>, ()> = Resource.ensure();
        let _r: Result<Absent<Resource>, ()> = Resource.ensure();
    }
}