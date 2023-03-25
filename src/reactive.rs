// use std::{marker::PhantomData, mem::MaybeUninit};

// use crate::{
//     copy_arena::RuntimeId,
//     signal::{with_rt, Signal, SignalEntry, SignalId},
// };

// trait Effect {
//     fn update(&mut self);
// }

// pub trait Rx<Reactive> {
//     type Owner<'a>
//     where
//         Self: 'a;

//     fn with<O, F: FnOnce(Reactive) -> O>(&mut self, runtime: RuntimeId, f: F) -> O {
//         let (signal, owner) = unsafe { self.borrow(runtime) };
//         let result = f(signal);
//         drop(owner);
//         result
//     }

//     /// # Safety
//     /// The caller must guarentee the returned value is dropped
//     unsafe fn borrow(&mut self, runtime: RuntimeId) -> (Reactive, Self::Owner<'_>);
// }

// impl<T: 'static> Rx<Signal<T>> for T {
//     type Owner<'a> = SignalOwner<'a, T>;

//     #[inline]
//     unsafe fn borrow(&mut self, runtime: RuntimeId) -> (Signal<T>, Self::Owner<'_>) {
//         let key = with_rt(runtime, |runtime| {
//             runtime.signals.borrow_mut().insert(SignalEntry {
//                 value: self,
//                 dependants: Default::default(),
//             })
//         });

//         let owner = SignalOwner {
//             runtime,
//             key: SignalId(key),
//             phantom: PhantomData,
//         };

//         let signal = Signal {
//             runtime,
//             key: SignalId(key),
//             phantom: PhantomData,
//         };

//         (signal, owner)
//     }
// }

// pub struct SignalOwner<'a, T: 'static> {
//     runtime: RuntimeId,
//     key: SignalId,
//     phantom: PhantomData<&'a mut T>,
// }

// impl<'a, T> Drop for SignalOwner<'a, T> {
//     fn drop(&mut self) {
//         with_rt(self.runtime, |rt| {
//             let mut signals = rt.signals.borrow_mut();
//             signals.remove(self.key.0);
//         });
//     }
// }

// impl<R: Rx<T>, T: 'static, const N: usize> Rx<[T; N]> for [R; N]
// where
//     [T; N]: Sized,
// {
//     type Owner<'a> = ArraySignalOwner<'a, T, R::Owner<'a>, N> where
//         Self: 'a;

//     unsafe fn borrow(&mut self, runtime: RuntimeId) -> ([T; N], Self::Owner<'_>) {
//         let mut to_drop: Vec<R::Owner<'_>> = Vec::with_capacity(N);
//         let mut signals: MaybeUninit<[T; N]> = MaybeUninit::uninit();

//         for (i, data) in self.iter_mut().enumerate() {
//             let (signal, owner) = data.borrow(runtime);

//             to_drop.push(owner);
//             signals.assume_init_mut()[i] = signal;
//         }

//         let to_drop: [R::Owner<'_>; N] = to_drop.try_into().unwrap_unchecked();
//         let signals: [T; N] = signals.assume_init();

//         let owner = ArraySignalOwner {
//             keys: to_drop,
//             phantom: PhantomData,
//         };

//         (signals, owner)
//     }
// }

// #[allow(unused)]
// pub struct ArraySignalOwner<'a, T, D, const N: usize> {
//     keys: [D; N],
//     phantom: PhantomData<&'a mut [T; N]>,
// }

// // impl<'a, R: Rx<T>, I: Iterator<Item = &'a mut R>, T: 'static> Rx<SignalIterator<'a, 'b, I, T>>
// //     for I
// // {
// //     type Owner<'b> =SignalIteratorOwner<'a, 'b, I, T> where
// //         Self: 'b;

// //     unsafe fn borrow(&mut self, runtime: RuntimeId) -> (SignalIterator<I, T>, Self::Owner<'_>) {
// //         let mut to_drop = Vec::new();

// //         let iter = SignalIterator {
// //             runtime,
// //             iter: self,
// //             to_drop: &mut to_drop,
// //         };

// //         let owner = SignalIteratorOwner {
// //             runtime,
// //             iter: self,
// //             to_drop: &mut to_drop,
// //         };

// //         (iter, owner)
// //     }
// // }

// // pub struct SignalIterator<'a, 'b, I: Iterator<Item = &'a mut T>, T: 'static> {
// //     runtime: RuntimeId,
// //     iter: I,
// //     to_drop: &'b mut Vec<SignalId>,
// // }

// // impl<'a, 'b, I: Iterator<Item = &'a mut T>, T: 'static> Iterator for SignalIterator<'a, 'b, I, T> {
// //     type Item = Signal<T>;

// //     fn next(&mut self) -> Option<Self::Item> {
// //         let next = self.iter.next()?;
// //         let key = with_rt(self.runtime, |runtime| {
// //             runtime.signals.borrow_mut().insert(SignalEntry {
// //                 value: next,
// //                 dependants: Default::default(),
// //             })
// //         });

// //         let signal = Signal {
// //             runtime: self.runtime,
// //             key: SignalId(key),
// //             phantom: PhantomData,
// //         };

// //         self.to_drop.push(SignalId(key));

// //         Some(signal)
// //     }
// // }

// // pub struct SignalIteratorOwner<'a, 'b, I: Iterator<Item = &'a mut T>, T: 'static> {
// //     runtime: RuntimeId,
// //     iter: I,
// //     to_drop: &'b mut Vec<SignalId>,
// // }
