/*!
Object implementing `Ensure` trait are in unknown inital external state and can be brought to a target state.

This can be seen as `TryInto` trait for objects with side effects with unknown initial external state and desired target state.

By calling `ensure()` we can be ensured that object is in its target state regardles if it was already in that state or had to be brought to it.
If object was already in target state nothing happens. Otherwise `ensure()` will call `meet()` on provided `EnsureAction` type to bring the object into its target state.

If object implements `Clone` method `ensure_verify()` can be used to make sure that object fulfills `Met` condition after `EnsureAction` type has been preformed.

Closures returning `Result<CheckEnsureResult, E>` that also return closure in `CheckEnsureResult::EnsureAction` variant automatically implement `Ensure` trait.
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
    Ok(if path.exists() {
        Met(())
    } else {
        EnsureAction(|| {
            File::create(&path).map(|file| drop(file))
        })
    })
}).expect("failed to create file");
```

# Existential types

This crate also provides `Present<T>` and `Absent<T>` wrapper types to mark ensured external states in type system.

If `T` implements `Ensure<Present<T>>` and `Ensure<Absetnt<T>>` it automatically implements `Existential<T>` trait
that provides methods `ensure_present()` and `ensure_absent()`.

See tests for example usage.
*/

use std::fmt;
use std::fmt::Debug;
use std::error::Error;
use std::ops::Deref;
use std::cmp::Ordering;

/// Result of verification if object is in target state with `check_ensure()`
#[derive(Debug)]
pub enum CheckEnsureResult<M, A> {
    Met(M),
    EnsureAction(A),
}

#[derive(Debug)]
pub struct VerificationError;

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "verification of target state failed after it was ensured to be met")
    }
}

/// Error raised if `Ensure::ensure_verify()` failed verification
impl Error for VerificationError {}

/// Function that can be used to bring object in its target state
pub trait Meet {
    type Met;
    type Error;

    fn meet(self) -> Result<Self::Met, Self::Error>;
}

/// Implement for types of objects that can be brought to target state `T`
pub trait Ensure<T>: Sized {
    type EnsureAction: Meet<Met = T>;

    /// Checks if target state is already `Met` or provides `EnsureAction` object which can by used to bring external state to target state by calling its `meet()` method
    fn check_ensure(self) -> Result<CheckEnsureResult<T, Self::EnsureAction>, <Self::EnsureAction as Meet>::Error>;

    /// Ensure target state by calling `check_ensure()` and if not `Met` calling `meet()` on `EnsureAction`
    fn ensure(self) -> Result<T, <Self::EnsureAction as Meet>::Error> {
        match self.check_ensure()? {
            CheckEnsureResult::Met(met) => Ok(met),
            CheckEnsureResult::EnsureAction(meet) => meet.meet(),
        }
    }

    /// Ensure target state and then verify that `EnsureAction` actually brought external state to target state by calling `check_ensure()` on clone of `self`
    fn ensure_verify(self) -> Result<T, <Self::EnsureAction as Meet>::Error> where Self: Clone, <Self::EnsureAction as Meet>::Error: From<VerificationError> {
        let verify = self.clone();
        match self.check_ensure()? {
            CheckEnsureResult::Met(met) => Ok(met),
            CheckEnsureResult::EnsureAction(action) => {
                let result = action.meet()?;
                match verify.check_ensure()? {
                    CheckEnsureResult::Met(_met) => Ok(result),
                    CheckEnsureResult::EnsureAction(_action) => Err(VerificationError.into()),
                }
            }
        }
    }
}

impl<T, E, A, F> Ensure<T> for F
where F: FnOnce() -> Result<CheckEnsureResult<T, A>, E>, A: Meet<Met = T, Error = E> {
    type EnsureAction = A;

    fn check_ensure(self) -> Result<CheckEnsureResult<T, Self::EnsureAction>, E> {
        self()
    }
}

impl<T, E, F> Meet for F
where F: FnOnce() -> Result<T, E> {
    type Met = T;
    type Error = E;

    fn meet(self) -> Result<Self::Met, Self::Error> {
        self()
    }
}

/// Run `ensure()` on object implementing `Ensure` and return its value.
/// This is useful with closures implementing `Ensure`.
pub fn ensure<T, E, R, A>(ensure: R) -> Result<T, E> where R: Ensure<T, EnsureAction = A>, A: Meet<Met = T, Error = E> {
    ensure.ensure()
}

/// Mark `T` as something that exists.
pub struct Present<T>(pub T);
/// Mark `T` as something that does not exist.
pub struct Absent<T>(pub T);

impl<T> Deref for Present<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Debug for Present<T> where T: Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Present")
            .field(&self.0)
            .finish()
    }
}

impl<T> PartialEq for Present<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialOrd for Present<T> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Deref for Absent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Debug for Absent<T> where T: Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Absent")
            .field(&self.0)
            .finish()
    }
}

impl<T> PartialEq for Absent<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialOrd for Absent<T> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Types implement `Existential` trait if they implement both `Ensure<Present<T>>` and `Ensure<Absent<T>>`.
pub trait Existential<T> {
    type Error;

    /// Ensure that `T` is `Present<T>`
    fn ensure_present(self) -> Result<Present<T>, Self::Error>;
    /// Ensure that `T` is `Absent<T>`
    fn ensure_absent(self) -> Result<Absent<T>, Self::Error>;
}

impl<T, E, R, PA, AA> Existential<T> for R where
    R: Ensure<Present<T>, EnsureAction = PA>, PA: Meet<Met = Present<T>, Error = E>,
    R: Ensure<Absent<T>, EnsureAction = AA>, AA: Meet<Met = Absent<T>, Error = E>
{
    type Error = E;

    fn ensure_present(self) -> Result<Present<T>, Self::Error> {
        self.ensure()
    }
    fn ensure_absent(self) -> Result<Absent<T>, Self::Error> {
        self.ensure()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::CheckEnsureResult::*;

    #[test]
    fn test_closure() {
        fn test(met: bool) -> impl Ensure<u8, EnsureAction = impl Meet<Met = u8, Error = ()>> {
            move || {
                Ok(match met {
                    true => Met(1),
                    _ => EnsureAction(|| Ok(2))
                })
            }
        }

        assert_eq!(test(true).ensure(), Ok(1));
        assert_eq!(test(false).ensure(), Ok(2));

        assert_eq!(ensure(test(true)), Ok(1));
        assert_eq!(ensure(test(false)), Ok(2));
    }

    struct Resource;

    struct CreateResourceAction(Resource);
    impl Meet for CreateResourceAction {
        type Met = Present<Resource>;
        type Error = ();

        fn meet(self) -> Result<Present<Resource>, ()> {
            Ok(Present(self.0))
        }
    }

    impl Ensure<Present<Resource>> for Resource {
        type EnsureAction = CreateResourceAction;

        fn check_ensure(self) -> Result<CheckEnsureResult<Present<Resource>, Self::EnsureAction>, ()> {
            Ok(if true {
                Met(Present(self))
            } else {
                EnsureAction(CreateResourceAction(self))
            })
        }
    }

    struct DeleteResourceAction(Resource);
    impl Meet for DeleteResourceAction {
        type Met = Absent<Resource>;
        type Error = ();

        fn meet(self) -> Result<Absent<Resource>, ()> {
            Ok(Absent(self.0))
        }
    }

    impl Ensure<Absent<Resource>> for Resource {
        type EnsureAction = DeleteResourceAction;

        fn check_ensure(self) -> Result<CheckEnsureResult<Absent<Resource>, Self::EnsureAction>, ()> {
            Ok(if true {
                Met(Absent(self))
            } else {
                EnsureAction(DeleteResourceAction(self))
            })
        }
    }

    #[test]
    fn test_ensure() {
        let _r: Result<Present<Resource>, ()> = Resource.ensure();
        let _r: Result<Absent<Resource>, ()> = Resource.ensure();
    }

    #[test]
    fn test_existential() {
        let _r: Result<Present<Resource>, ()> = Resource.ensure_present();
        let _r: Result<Absent<Resource>, ()> = Resource.ensure_absent();
    }
}
