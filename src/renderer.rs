use crate::{
    copy::{State, StateIO},
    events::{EventDescription, PlatformEvents},
};

pub trait Renderer<P: PlatformEvents>: Sized {
    fn node(&mut self) -> u32;

    fn append_all(&mut self, parent: u32, children: impl IntoIterator<Item = u32>);

    fn set_attribute(&mut self, id: u32, name: &'static str, value: &str);

    fn set_style(&mut self, id: u32, name: &'static str, value: &str);

    fn create_element(&mut self, id: u32, tag: &'static str);

    fn create_text(&mut self, id: u32, text: &str);

    fn set_text(&mut self, id: u32, text: &str);

    fn append_child(&mut self, parent: u32, child: u32);

    fn clone_node(&mut self, id: u32, new_id: u32);

    fn copy(&mut self, from: u32, to: u32);

    fn first_child(&mut self, id: u32);

    fn next_sibling(&mut self, id: u32);

    fn remove(&mut self, id: u32);

    fn return_node(&mut self, id: u32);

    fn add_listener<E: EventDescription<P>>(
        &mut self,
        id: u32,
        event: E,
        callback: Box<dyn FnMut(web_sys::Event)>,
    );

    fn flush(&mut self) {}
}

impl<R: Renderer<R> + PlatformEvents + Sized> Renderer<R> for State<R> {
    fn node(&mut self) -> u32 {
        self.with_mut(|r| r.node())
    }

    fn append_all(&mut self, parent: u32, children: impl IntoIterator<Item = u32>) {
        self.with_mut(|r| r.append_all(parent, children))
    }

    fn set_attribute(&mut self, id: u32, name: &'static str, value: &str) {
        self.with_mut(|r| r.set_attribute(id, name, value))
    }

    fn set_style(&mut self, id: u32, name: &'static str, value: &str) {
        self.with_mut(|r| r.set_style(id, name, value))
    }

    fn create_element(&mut self, id: u32, tag: &'static str) {
        self.with_mut(|r| r.create_element(id, tag))
    }

    fn create_text(&mut self, id: u32, text: &str) {
        self.with_mut(|r| r.create_text(id, text))
    }

    fn set_text(&mut self, id: u32, text: &str) {
        self.with_mut(|r| r.set_text(id, text))
    }

    fn append_child(&mut self, parent: u32, child: u32) {
        self.with_mut(|r| r.append_child(parent, child))
    }

    fn clone_node(&mut self, id: u32, new_id: u32) {
        self.with_mut(|r| r.clone_node(id, new_id))
    }

    fn copy(&mut self, from: u32, to: u32) {
        self.with_mut(|r| r.copy(from, to))
    }

    fn first_child(&mut self, id: u32) {
        self.with_mut(|r| r.first_child(id))
    }

    fn next_sibling(&mut self, id: u32) {
        self.with_mut(|r| r.next_sibling(id))
    }

    fn remove(&mut self, id: u32) {
        self.with_mut(|r| r.remove(id))
    }

    fn return_node(&mut self, id: u32) {
        self.with_mut(|r| r.return_node(id))
    }

    fn add_listener<E: EventDescription<R>>(
        &mut self,
        id: u32,
        event: E,
        callback: Box<dyn FnMut(web_sys::Event)>,
    ) {
        self.with_mut(|r| r.add_listener(id, event, callback))
    }

    fn flush(&mut self) {
        self.with_mut(|r| r.flush())
    }
}

impl<P: PlatformEvents> PlatformEvents for State<P> {
    type AnimationEvent = P::AnimationEvent;
    type BeforeUnloadEvent = P::BeforeUnloadEvent;
    type CompositionEvent = P::CompositionEvent;
    type DeviceMotionEvent = P::DeviceMotionEvent;
    type DeviceOrientationEvent = P::DeviceOrientationEvent;
    type DragEvent = P::DragEvent;
    type ErrorEvent = P::ErrorEvent;
    type Event = P::Event;
    type FocusEvent = P::FocusEvent;
    type GamepadEvent = P::GamepadEvent;
    type HashChangeEvent = P::HashChangeEvent;
    type InputEvent = P::InputEvent;
    type KeyboardEvent = P::KeyboardEvent;
    type MessageEvent = P::MessageEvent;
    type MouseEvent = P::MouseEvent;
    type PageTransitionEvent = P::PageTransitionEvent;
    type PointerEvent = P::PointerEvent;
    type PopStateEvent = P::PopStateEvent;
    type PromiseRejectionEvent = P::PromiseRejectionEvent;
    type SecurityPolicyViolationEvent = P::SecurityPolicyViolationEvent;
    type StorageEvent = P::StorageEvent;
    type SubmitEvent = P::SubmitEvent;
    type TouchEvent = P::TouchEvent;
    type TransitionEvent = P::TransitionEvent;
    type UiEvent = P::UiEvent;
    type WheelEvent = P::WheelEvent;
    type ProgressEvent = P::ProgressEvent;
}
