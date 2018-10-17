#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[derive(Debug)]
pub enum TryMet<M, U> {
    Met(M),
    MeetAction(U),
}

#[derive(Debug)]
pub enum Meet<N, M> {
    AlreadyMet(N),
    Met(M),
}

pub trait Meetable: Sized {
    type MeetAction: MeetAction;
    type Met;
    type Error: Into<<Self::MeetAction as MeetAction>::Error>;

    /// Chek if already Met or provide MeetAction
    fn try_met(self) -> Result<TryMet<Self::Met, Self::MeetAction>, Self::Error>;

    /// Cheks if AlreadMet or performs MeetAction and provides MeetAction::Met result
    fn meet(self) -> Result<Meet<Self::Met, <Self::MeetAction as MeetAction>::Met>, <Self::MeetAction as MeetAction>::Error> {
        match self.try_met() {
            Ok(TryMet::Met(met)) => Ok(Meet::AlreadyMet(met)),
            Err(err) => Err(err.into()),
            Ok(TryMet::MeetAction(meet)) => {
                match meet.meet() {
                    Ok(met) => Ok(Meet::Met(met)),
                    Err(err) => Err(err)
                }
            }
        }
    }
}

pub trait MeetAction {
    type Met;
    type Error;

    fn meet(self) -> Result<Self::Met, Self::Error>;
}

impl<MET, MA, IMF, METE> Meetable for IMF 
where IMF: FnOnce() -> Result<TryMet<MET, MA>, METE>, MA: MeetAction, METE: Into<<MA as MeetAction>::Error> {
    type MeetAction = MA;
    type Met = MET;
    type Error = METE;

    fn try_met(self) -> Result<TryMet<Self::Met, Self::MeetAction>, Self::Error> {
        self()
    }
}

impl<MET, MF, MEETE> MeetAction for MF
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
    use super::Meet::*;

    #[test]
    fn closure() {
        fn promise(met: bool, fail: bool) -> impl Meetable<Met = u8, MeetAction = impl MeetAction<Met = u16, Error = i16>, Error = i8> {
            move || {
                match (met, fail) {
                    (true, false) => Ok(TryMet::Met(1)),
                    (true, true) => Err(2),
                    _ => Ok(TryMet::MeetAction(move || match fail {
                        false => Ok(3),
                        true => Err(4),
                    }))
                }
            }
        }

        assert_matches!(promise(true, false).meet(), Ok(AlreadyMet(1u8)));
        assert_matches!(promise(true, true).meet(), Err(2i16));
        assert_matches!(promise(false, false).meet(), Ok(Met(3u16)));
        assert_matches!(promise(false, true).meet(), Err(4i16));
    }
}