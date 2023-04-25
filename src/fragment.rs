use crate::{
    component::{ComponentState, DynComponentState},
    prelude::{PlatformEvents, Renderer},
};

pub struct Fragment<R: Renderer<P>, P: PlatformEvents> {
    items: Vec<DynComponentState<R, P>>,
}

impl<R, P> Fragment<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    pub fn new(items: Vec<DynComponentState<R, P>>) -> Self {
        Self { items }
    }

    pub fn update(
        &mut self,
        iter: impl Iterator<Item = DynComponentState<R, P>>,
        parent: u32,
        ui: &mut R,
    ) {
        for old in self.items.drain(..) {
            old.remove(ui);
        }
        self.items = iter.collect();
        for new in &self.items {
            ui.append_all(parent, new.roots());
        }
    }
}

impl<R, P> ComponentState<R, P> for Fragment<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    fn roots(&self) -> Vec<u32> {
        self.items.iter().flat_map(|item| item.roots()).collect()
    }
}
