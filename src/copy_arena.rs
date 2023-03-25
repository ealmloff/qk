// use std::cell::RefCell;

// use slotmap::{DefaultKey, SlotMap};

// #[cfg(feature = "ssr")]
// new_key_type! { struct RuntimeId; }

// #[cfg(not(feature = "ssr"))]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
// pub struct RuntimeId;

// pub(crate) struct PtrArena<T> {
//     data: SlotMap<DefaultKey, RefCell<T>>,
// }

// impl<T> PtrArena<T> {
//     #[inline]
//     pub fn insert(&mut self, value: T) -> DefaultKey {
//         self.data.insert(RefCell::new(value))
//     }

//     #[inline]
//     pub fn insert_with_key<F: FnOnce(DefaultKey) -> T>(&mut self, f: F) -> DefaultKey {
//         let key = self.data.insert_with_key(|key| RefCell::new(f(key)));
//         key
//     }

//     #[inline]
//     pub fn remove(&mut self, id: DefaultKey) -> T {
//         let data = self
//             .data
//             .remove(id)
//             .expect("tried to remove a dropped item");
//         data.into_inner()
//     }

//     #[inline]
//     pub fn with_raw<O>(&self, id: DefaultKey, f: impl FnOnce(&RefCell<T>) -> O) -> O {
//         let data = self.data.get(id).expect("tried to get a dropped item");
//         f(data)
//     }

//     #[inline]
//     pub fn with_mut<F: FnOnce(&mut T) -> O, O>(&self, id: DefaultKey, f: F) -> O {
//         let data = self.data.get(id).expect("tried to get a dropped item");
//         let mut data = data.borrow_mut();
//         f(&mut data)
//     }
// }

// impl<T> Default for PtrArena<T> {
//     fn default() -> Self {
//         Self {
//             data: SlotMap::default(),
//         }
//     }
// }
