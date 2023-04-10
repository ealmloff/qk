use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use core::slice;
use std::fmt::{Display, Formatter, Write};
use std::hint::black_box;

pub use arguments_macro::*;

#[cfg(feature = "web")]
mod to_js;

const fn min_size(slice: &'static [&'static str]) -> usize {
    let mut idx = 0;
    let mut size = 0;
    while idx < slice.len() {
        let s = slice[idx];
        size += s.len();
        idx += 1;
    }
    size
}

#[derive(Debug, Clone, Copy)]
pub struct DiffableArguments<'a> {
    pub static_segments: &'static [&'static str],
    pub dynamic_segments: &'a [Entry<'a>],
}

impl<'a> DiffableArguments<'a> {
    pub fn to_str(&self) -> Option<&'a str> {
        if let DiffableArguments {
            static_segments: ["", ""],
            dynamic_segments: [Entry::Str(s)],
        } = self
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn to_bump_str(self, bump: &Bump) -> bumpalo::collections::String {
        let mut bump_str =
            bumpalo::collections::String::with_capacity_in(min_size(self.static_segments), bump);
        for (static_seg, dynamic_seg) in self
            .static_segments
            .iter()
            .zip(self.dynamic_segments.iter())
        {
            bump_str.write_str(static_seg).unwrap();
            match dynamic_seg {
                Entry::U32(u) => {
                    u.write(unsafe { bump_str.as_mut_vec() });
                }
                Entry::I32(i) => {
                    i.write(unsafe { bump_str.as_mut_vec() });
                }
                Entry::Bool(b) => match b {
                    true => {
                        bump_str.write_str("true").unwrap();
                    }
                    false => {
                        bump_str.write_str("false").unwrap();
                    }
                },
                Entry::Str(s) => bump_str.write_str(s).unwrap(),
            }
        }
        bump_str
            .write_str(self.static_segments.last().unwrap())
            .unwrap();
        bump_str
    }
}

#[test]
fn displays() {
    let bump = Bump::new();
    for num in 0..10000 {
        let diffable = DiffableArguments {
            static_segments: &["hello ", ", ", " welcome"],
            dynamic_segments: &[
                (&mut &"world").into_entry(&bump),
                (&mut &num).into_entry(&bump),
            ],
        };
        let bump = Bump::new();
        let string = diffable.to_bump_str(&bump);
        assert_eq!(string, format!("hello world, {num} welcome"));
    }
}

impl PartialEq for DiffableArguments<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.static_segments, other.static_segments)
            && self.dynamic_segments == other.dynamic_segments
    }
}

#[derive(Debug)]
pub enum Entry<'a> {
    U32(u32),
    I32(i32),
    Bool(bool),
    Str(&'a str),
}

pub trait IntoEntry<'a> {
    fn into_entry(self, bump: &'a Bump) -> Entry<'a>;
}

impl<'a, T: Display> IntoEntry<'a> for &T {
    #[inline(always)]
    fn into_entry(self, bump: &'a Bump) -> Entry<'a> {
        Entry::Str(bumpalo::format!(in bump, "{}", self).into_bump_str())
    }
}

impl<'a> IntoEntry<'a> for &mut &&'a str {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Str(self)
    }
}

impl<'a> IntoEntry<'a> for &mut &u32 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U32(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &u16 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U32(**self as u32)
    }
}

impl<'a> IntoEntry<'a> for &mut &u8 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::U32(**self as u32)
    }
}

impl<'a> IntoEntry<'a> for &mut &i32 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I32(**self)
    }
}

impl<'a> IntoEntry<'a> for &mut &i16 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I32(**self as i32)
    }
}

impl<'a> IntoEntry<'a> for &mut &i8 {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::I32(**self as i32)
    }
}

impl<'a> IntoEntry<'a> for &mut &bool {
    #[inline(always)]
    fn into_entry(self, _bump: &'a Bump) -> Entry<'a> {
        Entry::Bool(**self)
    }
}

impl PartialEq for Entry<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::I32(l0), Self::I32(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => std::ptr::eq(*l0, *r0) || l0 == r0,
            _ => false,
        }
    }
}

#[test]
fn bench_num() {
    for x in [0, 100, 1000, 10000] {
        let y = black_box(x);
        let x = black_box(x);
        bench1(x, y);
        bench2(x, y);
    }
    #[inline(never)]
    fn bench1(x: u64, y: u64) {
        let mut bump = Bump::new();
        let static_segments = &["a", "b", "c", ""];
        for _ in 0..10000000 {
            let d1 = DiffableArguments {
                static_segments,
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            let d2 = DiffableArguments {
                static_segments,
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d1 == d2);
            bump.reset()
        }
    }

    #[inline(never)]
    fn bench2(x: u64, y: u64) {
        let mut bump = Bump::new();
        for _ in 0..10000000 {
            let d1 = bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3).into_bump_str();
            let d2 = bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3).into_bump_str();
            black_box(d1 == d2);
            bump.reset();
        }
    }
}

#[test]
fn bench_str() {
    let mut y;
    for x in ["hello", "world", "this", "test"] {
        y = x;
        bench1(x, y);
        bench2(x, y);
    }
    #[inline(never)]
    fn bench1(x: &str, y: &str) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            let d1 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            let d2 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d1 == d2);
        }
    }

    #[inline(never)]
    fn bench2(x: &str, y: &str) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            let d1 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            let d2 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d1 == d2);
        }
    }
}

#[test]
fn bench_fmted() {
    struct Testing;
    impl Display for Testing {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "hello")
        }
    }
    bench1(Testing, Testing);
    bench2(Testing, Testing);

    #[inline(never)]
    fn bench1(x: Testing, y: Testing) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            let d1 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            let d2 = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d1 == d2);
        }
    }

    #[inline(never)]
    fn bench2(x: Testing, y: Testing) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            let d1 = bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3).into_bump_str();
            let d2 = bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3).into_bump_str();
            black_box(d1 == d2);
        }
    }
}

#[test]
fn bench_write() {
    for x in 0..10 {
        bench1(x, x);
        bench2(x, x);
    }

    #[inline(never)]
    fn bench1(x: usize, y: usize) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            let d = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d.to_bump_str(&bump));
        }
    }

    #[inline(never)]
    fn bench2(x: usize, y: usize) {
        for _ in 0..1000000 {
            let bump = Bump::new();
            black_box(bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3));
        }
    }
}

#[test]
fn bench_write_fmted() {
    struct Testing;
    impl Display for Testing {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "hello")
        }
    }
    bench1(Testing, Testing);
    bench2(Testing, Testing);

    #[inline(never)]
    fn bench1(x: Testing, y: Testing) {
        for _ in 0..10000000 {
            let bump = Bump::new();
            let d = DiffableArguments {
                static_segments: &["a", "b", "c"],
                dynamic_segments: bump.alloc_with(|| {
                    [
                        (&mut &x).into_entry(&bump),
                        (&mut &y).into_entry(&bump),
                        (&mut &3u64).into_entry(&bump),
                    ]
                }),
            };
            black_box(d.to_bump_str(&bump));
        }
    }

    #[inline(never)]
    fn bench2(x: Testing, y: Testing) {
        for _ in 0..10000000 {
            let bump = Bump::new();
            black_box(bumpalo::format!(in &bump, "a{}b{}c{}", x, y, 3));
        }
    }
}

pub trait Writable {
    fn write(self, into: &mut BumpVec<u8>);
}

macro_rules! write_unsized {
    ($t: ty) => {
        impl Writable for $t {
            fn write(self, to: &mut BumpVec<u8>) {
                let mut n = self;
                let mut n2 = n;
                let mut num_digits = 0;
                while n2 > 0 {
                    n2 /= 10;
                    num_digits += 1;
                }
                let len = num_digits.max(1);
                to.reserve(len);
                let ptr = to.as_mut_ptr().cast::<u8>();
                let old_len = to.len();
                let mut i = len - 1;
                loop {
                    unsafe { ptr.add(old_len + i).write((n % 10) as u8 + b'0') }
                    n /= 10;

                    if n == 0 {
                        break;
                    } else {
                        i -= 1;
                    }
                }

                #[allow(clippy::uninit_vec)]
                unsafe {
                    to.set_len(old_len + (len - i));
                }
            }
        }
    };
}

macro_rules! write_sized {
    ($t: ty) => {
        impl Writable for $t {
            fn write(self, to: &mut BumpVec<u8>) {
                let neg = self < 0;
                let mut n = if neg {
                    match self.checked_abs() {
                        Some(n) => n,
                        None => <$t>::MAX / 2 + 1,
                    }
                } else {
                    self
                };
                let mut n2 = n;
                let mut num_digits = 0;
                while n2 > 0 {
                    n2 /= 10;
                    num_digits += 1;
                }
                num_digits = num_digits.max(1);
                let len = if neg { num_digits + 1 } else { num_digits };
                to.reserve(len);
                let ptr = to.as_mut_ptr().cast::<u8>();
                let old_len = to.len();
                let mut i = len - 1;
                loop {
                    unsafe { ptr.add(old_len + i).write((n % 10) as u8 + b'0') }
                    n /= 10;

                    if n == 0 {
                        break;
                    } else {
                        i -= 1;
                    }
                }

                if neg {
                    i -= 1;
                    unsafe { ptr.add(old_len + i).write(b'-') }
                }

                #[allow(clippy::uninit_vec)]
                unsafe {
                    to.set_len(old_len + (len - i));
                }
            }
        }
    };
}

write_unsized!(u8);
write_unsized!(u16);
write_unsized!(u32);
write_unsized!(u64);
write_unsized!(u128);
write_unsized!(usize);

write_sized!(i8);
write_sized!(i16);
write_sized!(i32);
write_sized!(i64);
write_sized!(i128);
write_sized!(isize);
