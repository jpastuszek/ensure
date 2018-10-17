#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[derive(Debug)]
pub enum IsMetResult<M, U> {
    IsMet(M),
    IsUnmet(U),
}

#[derive(Debug)]
pub enum MeetResult<N, M> {
    NothingToDo(N),
    NowMet(M),
}

use IsMetResult::*;
use MeetResult::*;

pub trait Meetable: Sized {
    type Meet: Meet;
    type Met;
    type Error: Into<<Self::Meet as Meet>::Error>;

    fn is_met(self) -> Result<IsMetResult<Self::Met, Self::Meet>, Self::Error>;

    fn meet(self) -> Result<MeetResult<Self::Met, <Self::Meet as Meet>::Met>, <Self::Meet as Meet>::Error> {
        match self.is_met() {
            Ok(IsMet(met)) => Ok(NothingToDo(met)),
            Err(err) => Err(err.into()),
            Ok(IsUnmet(meet)) => {
                match meet.meet() {
                    Ok(met) => Ok(NowMet(met)),
                    Err(err) => Err(err)
                }
            }
        }
    }
}

pub trait Meet {
    type Met;
    type Error;

    fn meet(self) -> Result<Self::Met, Self::Error>;
}

impl<MET, MEET, IMF, METE> Meetable for IMF 
where IMF: FnOnce() -> Result<IsMetResult<MET, MEET>, METE>, MEET: Meet, METE: Into<<MEET as Meet>::Error> {
    type Meet = MEET;
    type Met = MET;
    type Error = METE;

    fn is_met(self) -> Result<IsMetResult<Self::Met, Self::Meet>, Self::Error> {
        self()
    }
}

impl<MET, MF, MEETE> Meet for MF
where MF: FnOnce() -> Result<MET, MEETE> {
    type Met = MET;
    type Error = MEETE;

    fn meet(self) -> Result<Self::Met, Self::Error> {
        self()
    }
}

pub struct Existing<T>(pub T);
pub struct NonExisting<T>(pub T);

pub trait Existential: Sized {
    fn assume_existing(self) -> Existing<Self> {
        Existing(self)
    }

    fn assume_non_existing(self) -> NonExisting<Self> {
        NonExisting(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn closure() {
        fn promise(met: bool, fail: bool) -> impl Meetable<Met = u8, Meet = impl Meet<Met = u16, Error = i16>, Error = i8> {
            move || {
                match (met, fail) {
                    (true, false) => Ok(IsMet(1)),
                    (true, true) => Err(2),
                    _ => Ok(IsUnmet(move || match fail {
                        false => Ok(3),
                        true => Err(4),
                    }))
                }
            }
        }

        assert_matches!(promise(true, false).meet(), Ok(NothingToDo(1u8)));
        assert_matches!(promise(true, true).meet(), Err(2i16));
        assert_matches!(promise(false, false).meet(), Ok(NowMet(3u16)));
        assert_matches!(promise(false, true).meet(), Err(4i16));
    }
}