use std::{cell::RefCell, rc::Rc};

use crate::prelude::{PlatformEvents, Renderer};

pub trait Component<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    type State: ComponentState<R, P>;

    fn create(self, ui: &mut R) -> Self::State;
}

pub trait ComponentState<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    fn roots(&self) -> Vec<u32>;

    fn remove(&self, ui: &mut R) {
        for root in self.roots() {
            ui.remove(root);
        }
    }
}

impl<R, P, C> ComponentState<R, P> for Rc<RefCell<C>>
where
    C: ComponentState<R, P>,
    R: Renderer<P>,
    P: PlatformEvents,
{
    fn roots(&self) -> Vec<u32> {
        self.borrow().roots()
    }
}

pub struct DynComponentState<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    inner: Box<dyn ComponentState<R, P>>,
}

impl<R, P> DynComponentState<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    pub fn new<C: ComponentState<R, P> + 'static>(inner: C) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl<R, P> ComponentState<R, P> for DynComponentState<R, P>
where
    R: Renderer<P>,
    P: PlatformEvents,
{
    fn roots(&self) -> Vec<u32> {
        self.inner.roots()
    }
}
