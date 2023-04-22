use crate::events::{EventDescription, PlatformEvents};

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

impl<'a, R: Renderer<R> + PlatformEvents + Sized> Renderer<R> for &'a mut R {
    fn node(&mut self) -> u32 {
        R::node(self)
    }

    fn append_all(&mut self, parent: u32, children: impl IntoIterator<Item = u32>) {
        R::append_all(self, parent, children)
    }

    fn set_attribute(&mut self, id: u32, name: &'static str, value: &str) {
        R::set_attribute(self, id, name, value)
    }

    fn set_style(&mut self, id: u32, name: &'static str, value: &str) {
        R::set_style(self, id, name, value)
    }

    fn create_element(&mut self, id: u32, tag: &'static str) {
        R::create_element(self, id, tag)
    }

    fn create_text(&mut self, id: u32, text: &str) {
        R::create_text(self, id, text)
    }

    fn set_text(&mut self, id: u32, text: &str) {
        R::set_text(self, id, text)
    }

    fn append_child(&mut self, parent: u32, child: u32) {
        R::append_child(self, parent, child)
    }

    fn clone_node(&mut self, id: u32, new_id: u32) {
        R::clone_node(self, id, new_id)
    }

    fn copy(&mut self, from: u32, to: u32) {
        R::copy(self, from, to)
    }

    fn first_child(&mut self, id: u32) {
        R::first_child(self, id)
    }

    fn next_sibling(&mut self, id: u32) {
        R::next_sibling(self, id)
    }

    fn remove(&mut self, id: u32) {
        R::remove(self, id)
    }

    fn return_node(&mut self, id: u32) {
        R::return_node(self, id)
    }

    fn add_listener<E: EventDescription<R>>(
        &mut self,
        id: u32,
        event: E,
        callback: Box<dyn FnMut(web_sys::Event)>,
    ) {
        R::add_listener(self, id, event, callback)
    }

    fn flush(&mut self) {
        R::flush(self)
    }
}
