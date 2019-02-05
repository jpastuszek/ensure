/*!
Object implementing `Ensure` trait are in unknown inital state and can be brought to a target state.

By calling `met()` object can be ensured to be in its target state.
If object was already in target state nothing happens. Otherwise `met()` will call `meet()` on provided `MeetAction` to bring the object into its target state.

If object implements `Clone` method `met_verify()` can be used to make sure that object fulfills `Met` condition after `MeetAction` has been preformed.
*/

use std::fmt;
use std::error::Error;

/// Result of verification if object is in target state with `try_met()`
#[derive(Debug)]
pub enum TryMetResult<M, U> {
    Met(M),
    MeetAction(U),
}

/// Result of ensuring target state with `met()`
#[derive(Debug)]
pub enum MetResult<N, M> {
    AlreadyMet(N),
    Met(M),
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
    type MeetAction: MeetAction;
    type Met;

    /// Check if already `Met` or provide `MeetAction` which can be performed by calling `meet()`
    fn try_met(self) -> TryMetResult<Self::Met, Self::MeetAction>;

    /// Meet the Ensure by calling `try_met()` and if not `Met` calling `meet()` on `MeetAction`
    fn met(self) -> MetResult<Self::Met, <Self::MeetAction as MeetAction>::Met> {
        match self.try_met() {
            TryMetResult::Met(met) => MetResult::AlreadyMet(met),
            TryMetResult::MeetAction(meet) => MetResult::Met(meet.meet()),
        }
    }

    /// Ensure it is `met()` and then verify it is in fact `Met` with `try_met()`
    fn met_verify(self) -> Result<Self::Met, UnmetError> where Self: Clone {
        let verify = self.clone();
        match self.met() {
            MetResult::AlreadyMet(met) => Ok(met),
            MetResult::Met(_action_met) => match verify.try_met() {
                TryMetResult::Met(met) => Ok(met),
                TryMetResult::MeetAction(_action) => Err(UnmetError),
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
where IMF: FnOnce() -> TryMetResult<MET, MA>, MA: MeetAction {
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
    use super::MetResult::*;
    use assert_matches::*;

    #[test]
    fn closure() {
        fn ensure(met: bool) -> impl Ensure<Met = u8, MeetAction = impl MeetAction<Met = u16>> {
            move || {
                match met {
                    true => TryMetResult::Met(1),
                    _ => TryMetResult::MeetAction(move || 2)
                }
            }
        }

        assert_matches!(ensure(true).met(), AlreadyMet(1));
        assert_matches!(ensure(false).met(), Met(2));
    }
}