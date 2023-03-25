// use rustc_hash::FxHashSet;
// use slotmap::DefaultKey;
// use std::{
//     any::Any,
//     cell::RefCell,
//     fmt::{Debug, Display},
//     marker::PhantomData,
// };

// use crate::copy_arena::{PtrArena, RuntimeId};

// #[cfg(feature = "ssr")]
// thread_local! {
//     static RUNTIMES: RefCell<slotmap::SlotMap<RuntimeId, Runtime>> = RefCell::new(SlotMap::default());
// }

// #[cfg(not(feature = "ssr"))]
// thread_local! {
//     static RUNTIME: Runtime = Runtime::new();
// }

// impl RuntimeId {
//     pub fn create() -> Self {
//         #[cfg(feature = "ssr")]
//         return RUNTIMES.with(|runtimes| {
//             let mut runtimes = runtimes.borrow_mut();
//             runtimes.insert(Runtime::new())
//         });
//         #[cfg(not(feature = "ssr"))]
//         return RuntimeId;
//     }

//     // T is a mutable reference that is lended into the arena.
//
//     pub fn with_signal<T: 'static, O>(self, data: &mut T, f: impl FnOnce(Signal<T>) -> O) -> O {
//         let key = with_rt(self, |runtime| {
//             runtime.signals.borrow_mut().insert(SignalEntry {
//                 value: data,
//                 dependants: Default::default(),
//             })
//         });
//         let r = f(Signal {
//             runtime: self,
//             key: SignalId(key),
//             phantom: PhantomData,
//         });
//         with_rt(self, |runtime| {
//             runtime.signals.borrow_mut().remove(key);
//         });
//         r
//     }

//     // T is a mutable reference that is lended into the arena.
//
//     pub fn with_effect<O>(self, data: &mut (impl FnMut() + 'static), f: impl FnOnce() -> O) -> O {
//         let key = with_rt(self, |runtime| {
//             let key = runtime.effects.borrow_mut().insert(data as *mut _);
//             runtime.run_effect(EffectId(key));
//             key
//         });

//         let r = f();

//         with_rt(self, |runtime| {
//             runtime.effects.borrow_mut().remove(key);
//         });

//         r
//     }
// }

// #[inline]
// pub(crate) fn with_rt<O>(runtime_id: RuntimeId, f: impl FnOnce(&Runtime) -> O) -> O {
//     #[cfg(not(feature = "ssr"))]
//     {
//         let _ = runtime_id;
//         RUNTIME.with(f)
//     }
//     #[cfg(feature = "ssr")]
//     RUNTIMES.with(|runtimes| {
//         let runtimes = runtimes.borrow();
//         let runtime = runtimes
//             .get(runtime_id)
//             .expect("tried to get a runtime that was dropped");
//         f(runtime)
//     })
// }

// /// Provide the runtime for signals
// ///
// /// This will reuse dead runtimes
// pub fn claim_rt() -> RuntimeId {
//     #[cfg(not(feature = "ssr"))]
//     return RuntimeId;
//     #[cfg(feature = "ssr")]
//     RUNTIMES.with(|runtimes| runtimes.borrow_mut().insert(Runtime::new()))
// }

// /// Removes the runtime from the thread local storage
// /// This will drop all signals and effects
// pub fn drop_rt(runtime_id: RuntimeId) {
//     #[cfg(not(feature = "ssr"))]
//     let _ = runtime_id;
//     #[cfg(feature = "ssr")]
//     RUNTIMES.with(|runtimes| {
//         runtimes.borrow_mut().remove(runtime_id);
//     });
// }

// pub struct Runtime {
//     effects: RefCell<PtrArena<*mut dyn FnMut()>>,
//     pub(crate) signals: RefCell<PtrArena<SignalEntry>>,
//     effect_stack: RefCell<Vec<EffectId>>,
// }

// impl Runtime {
//     fn new() -> Self {
//         Self {
//             effect_stack: RefCell::new(Vec::new()),
//             effects: RefCell::new(PtrArena::default()),
//             signals: RefCell::new(PtrArena::default()),
//         }
//     }

//     #[inline]
//     fn with<T: 'static, F: FnOnce(&T) -> U, U>(&self, signal: Signal<T>, f: F) -> U {
//         let signals = self.signals.borrow();
//         signals.with_raw(signal.key.0, |signal| {
//             // add dependants
//             if let Some(effect) = self.effect_stack.borrow().last() {
//                 let mut borrowed = signal.borrow_mut();
//                 borrowed.dependants.insert(*effect);
//             }

//             let borrowed = signal.borrow();
//             // SAFETY: We know that the signal is valid because it still exists in the slotmap
//             let value = unsafe { &*borrowed.value };
//             f(value.downcast_ref::<T>().unwrap())
//         })
//     }

//     #[inline]
//     fn modify<T: 'static, F: FnOnce(&mut T)>(&self, signal: Signal<T>, f: F) {
//         let dependants = {
//             let signals = self.signals.borrow();
//             signals.with_mut(signal.key.0, |signal| {
//                 // SAFETY: We know that the signal is valid because it still exists in the slotmap
//                 let value = unsafe { &mut *signal.value };
//                 f(value.downcast_mut::<T>().unwrap());

//                 signal.dependants.clone()
//             })
//         };

//         // run dependants
//         for effect in dependants.iter() {
//             self.run_effect(*effect)
//         }
//     }

//     #[inline]
//     fn run_effect(&self, effect: EffectId) {
//         {
//             let mut stack = self.effect_stack.borrow_mut();
//             stack.push(effect);
//         }
//         let effects = self.effects.borrow();
//         effects.with_mut(effect.0, |effect| {
//             // SAFETY: We know that the effect is valid because it still exists in the slotmap
//             let f = unsafe { &mut **effect };
//             f();
//         });
//         self.effect_stack.borrow_mut().pop();
//     }
// }

// pub(crate) struct SignalEntry {
//     pub value: *mut dyn Any,
//     pub dependants: FxHashSet<EffectId>,
// }

// pub struct Signal<T: 'static> {
//     pub(crate) runtime: RuntimeId,
//     pub(crate) key: SignalId,
//     pub(crate) phantom: std::marker::PhantomData<T>,
// }

// impl<T: 'static> Default for Signal<T> {
//     fn default() -> Self {
//         Self {
//             runtime: RuntimeId,
//             key: SignalId::default(),
//             phantom: PhantomData,
//         }
//     }
// }

// impl<T: Display> Display for Signal<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.with(|x| x.fmt(f))
//     }
// }

// impl<T: Debug> Debug for Signal<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.with(|x| x.fmt(f))
//     }
// }

// impl<T: 'static> Clone for Signal<T> {
//     fn clone(&self) -> Self {
//         Self {
//             runtime: self.runtime,
//             key: self.key,
//             phantom: self.phantom,
//         }
//     }
// }

// impl<T: 'static> Copy for Signal<T> {}

// impl<T: 'static> Signal<T> {
//     #[inline]
//     pub fn set(&self, value: T) {
//         self.modify(|x| *x = value)
//     }

//     #[inline]
//     pub fn with<U: 'static, F: FnOnce(&T) -> U>(&self, f: F) -> U {
//         with_rt(self.runtime, |rt| rt.with(*self, f))
//     }

//     #[inline]
//     pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
//         with_rt(self.runtime, |rt| rt.modify(*self, f))
//     }
// }

// impl<T: 'static + Copy> Signal<T> {
//     #[inline]
//     pub fn get(&self) -> T {
//         self.with(|x| *x)
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
// pub(crate) struct SignalId(pub DefaultKey);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
// pub(crate) struct EffectId(DefaultKey);
