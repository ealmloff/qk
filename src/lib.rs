pub mod copy;
// pub mod copy_arena;
pub mod copy_ll;
pub mod events;
pub mod renderer;
pub(crate) mod slab;
pub mod web;

use std::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
};

pub use qk_macro;

// fn test() {
//     // let (rows, mut_rows): Rx<Vec<RowData>> = todo!();
//     // let y = 'rx: {
//     //     println!("0");
//     //     'rx: {
//     //         println!("1");
//     //     }
//     // };
//     // render! {
//     //     <For each={rows} as={row}>
//     //         <div>
//     //             <div>{row.name}</div>
//     //             <div>{row.age}</div>
//     //         </div>
//     //     </For>
//     // }
// }

struct Rx<'a, T> {
    data: &'a mut T,
    write: &'a mut bool,
}

impl<'a, T> Deref for Rx<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> DerefMut for Rx<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.write = true;
        self.data
    }
}
