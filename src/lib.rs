#[cfg(test)]
#[macro_use]
extern crate assert_matches;
extern crate either;
use either::Either;
use either::Either::*;

pub trait Promise: Sized {
    type Meet: Meet;
    type Met;
    type Error;

    fn is_met(self) -> Result<Either<Self::Met, Self::Meet>, Self::Error>;

    fn meet(self) -> Result<Either<Self::Met, <Self::Meet as Meet>::Met>, Either<Self::Error, <Self::Meet as Meet>::Error>> {
        match self.is_met() {
            Ok(Left(met)) => Ok(Left(met)),
            Err(err) => Err(Left(err)),
            Ok(Right(meet)) => {
                match meet.meet() {
                    Ok(met) => Ok(Right(met)),
                    Err(err) => Err(Right(err))
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

impl<MET, MEET, IMF, METE> Promise for IMF 
where IMF: FnOnce() -> Result<Either<MET, MEET>, METE>, MEET: Meet {
    type Meet = MEET;
    type Met = MET;
    type Error = METE;

    fn is_met(self) -> Result<Either<Self::Met, Self::Meet>, Self::Error> {
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
        fn promise(met: bool, fail: bool) -> impl Promise<Met = u8, Meet = impl Meet<Met = u16, Error = i16>, Error = i8> {
            move || {
                match (met, fail) {
                    (true, false) => Ok(Left(1)),
                    (true, true) => Err(2),
                    _ => Ok(Right(move || match fail {
                        false => Ok(3),
                        true => Err(4),
                    }))
                }
            }
        }

        assert_matches!(promise(true, false).meet(), Ok(Left(1u8)));
        assert_matches!(promise(true, true).meet(), Err(Left(2i8)));
        assert_matches!(promise(false, false).meet(), Ok(Right(3u16)));
        assert_matches!(promise(false, true).meet(), Err(Right(4i16)));
    }
}