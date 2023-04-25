// pub mod copy;
// pub mod copy_ll;
pub mod component;
pub mod events;
pub mod fragment;
pub mod prelude;
pub mod renderer;
pub(crate) mod slab;
mod tracking;
pub mod web;

use component::{Component, ComponentState};
use prelude::{PlatformEvents, Renderer};
pub use qk_macro;

pub fn launch<C, R: Renderer<R> + PlatformEvents + Sized>(mut ui: R, props: C)
where
    C: Component<R, R>,
{
    let comp = props.create(&mut ui);
    ui.append_all(0, comp.roots());
    ui.flush();
}
