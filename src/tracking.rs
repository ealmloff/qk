use num_traits::PrimInt;
use std::cell::Cell;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct DirtyTrackSet<R, W> {
    pub read: Cell<R>,
    pub write: Cell<W>,
}

impl<R: PrimInt, W: PrimInt> DirtyTrackSet<R, W> {
    pub fn is_read(&self, num: u8) -> bool {
        !(self.read.get() & (R::one() << num as usize)).is_zero()
    }

    pub fn is_write(&self, num: u8) -> bool {
        !(self.write.get() & (W::one() << num as usize)).is_zero()
    }

    pub fn track(&self, num: u8) -> DirtyTrack<R, W> {
        DirtyTrack { data: self, num }
    }

    pub fn get_read(&self) -> R {
        self.read.get()
    }

    pub fn reset_read(&self) {
        self.read.set(R::zero());
    }

    pub fn get_write(&self) -> W {
        self.write.get()
    }

    pub fn reset_write(&self) {
        self.write.set(W::zero());
    }
}

#[derive(Copy, Clone)]
pub struct DirtyTrack<'a, R, W> {
    pub data: &'a DirtyTrackSet<R, W>,
    pub num: u8,
}

impl<R: PrimInt, W: PrimInt> DirtyTrack<'_, R, W> {
    fn read(&self) {
        self.data
            .read
            .set(self.data.read.get() | (R::one() << self.num as usize));
    }

    fn write(&self) {
        self.data
            .write
            .set(self.data.write.get() | (W::one() << self.num as usize));
    }
}

pub struct RwTrack<'a, T, R, W> {
    pub data: &'a mut T,
    pub tracking: DirtyTrack<'a, R, W>,
}

impl<T: Display, R: PrimInt, W: PrimInt> Display for RwTrack<'_, T, R, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl<T: Debug, R: PrimInt, W: PrimInt> Debug for RwTrack<'_, T, R, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl<T, R: PrimInt, W: PrimInt> Deref for RwTrack<'_, T, R, W> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.tracking.read();
        self.data
    }
}

impl<T, R: PrimInt, W: PrimInt> DerefMut for RwTrack<'_, T, R, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tracking.write();
        self.data
    }
}

#[test]
fn rw_track() {
    let mut value = 0;
    let tracking: DirtyTrackSet<u8, u8> = DirtyTrackSet::default();

    {
        let mut value = RwTrack {
            data: &mut value,
            tracking: tracking.track(0),
        };

        if *value == 0 {
            *value = 1;
        }

        assert!(tracking.is_write(0));
    }

    tracking.reset_write();

    let mut value1 = 0;
    let mut value2 = 0;

    {
        let value1 = RwTrack {
            data: &mut value1,
            tracking: tracking.track(0),
        };
        let mut value2 = RwTrack {
            data: &mut value2,
            tracking: tracking.track(1),
        };

        if *value1 == 0 {
            *value2 = 1;
        }

        assert!(!tracking.is_write(0));
        assert!(tracking.is_write(1));
    }
}

pub struct Effect<F, T> {
    pub rx: F,
    pub rx_subscriptions: u8,
    pub current: T,
}
