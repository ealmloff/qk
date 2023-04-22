use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::slab::IdSlab;
use crate::{
    events::{EventDescription, PlatformEvents, EVENT_COUNT},
    renderer::Renderer,
};

#[derive(Clone)]
pub struct WebRenderer(Rc<RefCell<WebRendererInner>>);

pub struct WebRendererInner {
    channel: Channel,
    ids: IdSlab<()>,
    queued_listeners: Vec<(u32, &'static str, Box<dyn FnMut(web_sys::Event)>)>,
    event_handlers: SharedListeners,
}

impl PlatformEvents for WebRenderer {
    type AnimationEvent = web_sys::AnimationEvent;
    type BeforeUnloadEvent = web_sys::BeforeUnloadEvent;
    type CompositionEvent = web_sys::CompositionEvent;
    type DeviceMotionEvent = web_sys::DeviceMotionEvent;
    type DeviceOrientationEvent = web_sys::DeviceOrientationEvent;
    type DragEvent = web_sys::DragEvent;
    type ErrorEvent = web_sys::ErrorEvent;
    type FocusEvent = web_sys::FocusEvent;
    type GamepadEvent = web_sys::GamepadEvent;
    type HashChangeEvent = web_sys::HashChangeEvent;
    type InputEvent = web_sys::InputEvent;
    type KeyboardEvent = web_sys::KeyboardEvent;
    type MessageEvent = web_sys::MessageEvent;
    type MouseEvent = web_sys::MouseEvent;
    type PageTransitionEvent = web_sys::PageTransitionEvent;
    type PointerEvent = web_sys::PointerEvent;
    type PopStateEvent = web_sys::PopStateEvent;
    type PromiseRejectionEvent = web_sys::PromiseRejectionEvent;
    type SecurityPolicyViolationEvent = web_sys::SecurityPolicyViolationEvent;
    type StorageEvent = web_sys::StorageEvent;
    type SubmitEvent = web_sys::SubmitEvent;
    type TouchEvent = web_sys::TouchEvent;
    type TransitionEvent = web_sys::TransitionEvent;
    type UiEvent = web_sys::UiEvent;
    type WheelEvent = web_sys::WheelEvent;
    type ProgressEvent = web_sys::ProgressEvent;
    type Event = web_sys::Event;
}

impl Default for WebRenderer {
    fn default() -> Self {
        let mut ids: IdSlab<()> = IdSlab::default();

        // the root node
        ids.id(());

        Self(Rc::new(RefCell::new(WebRendererInner {
            channel: Channel::default(),
            ids,
            queued_listeners: Vec::new(),
            event_handlers: SharedListeners::default(),
        })))
    }
}

impl Renderer<WebRenderer> for WebRenderer {
    fn node(&mut self) -> u32 {
        let mut myself = self.0.borrow_mut();
        myself.ids.id(())
    }

    fn append_all(&mut self, parent: u32, children: impl IntoIterator<Item = u32>) {
        for child in children.into_iter() {
            self.append_child(parent, child);
        }
    }

    fn set_attribute(&mut self, id: u32, name: &'static str, value: &str) {
        let mut myself = self.0.borrow_mut();
        myself.channel.set_attribute(id, name, value);
    }

    fn set_style(&mut self, id: u32, name: &'static str, value: &str) {
        let mut myself = self.0.borrow_mut();
        myself.channel.set_style(id, name, value);
    }

    fn create_element(&mut self, id: u32, tag: &'static str) {
        let mut myself = self.0.borrow_mut();
        myself.channel.create_element(id, tag);
    }

    fn create_text(&mut self, id: u32, text: &str) {
        let mut myself = self.0.borrow_mut();
        myself.channel.create_text(id, text);
    }

    fn set_text(&mut self, id: u32, text: &str) {
        let mut myself = self.0.borrow_mut();
        myself.channel.set_text(id, text);
    }

    fn append_child(&mut self, parent: u32, child: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.append_child(parent, child);
    }

    fn clone_node(&mut self, id: u32, new_id: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.clone(id, new_id);
    }

    fn copy(&mut self, id: u32, id2: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.copy(id, id2);
    }

    fn first_child(&mut self, id: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.first_child(id);
    }

    fn next_sibling(&mut self, id: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.next_sibling(id);
    }

    fn remove(&mut self, id: u32) {
        let mut myself = self.0.borrow_mut();
        myself.channel.remove(id);
    }

    fn return_node(&mut self, id: u32) {
        let mut myself = self.0.borrow_mut();
        myself.ids.recycle(id)
    }

    fn add_listener<E: EventDescription<WebRenderer>>(
        &mut self,
        id: u32,
        _: E,
        callback: Box<dyn FnMut(web_sys::Event)>,
    ) {
        let mut myself = self.0.borrow_mut();
        let event_name = E::NAME;

        if E::BUBBLES {
            let listeners = myself.event_handlers.clone();
            {
                let handler_id = {
                    let mut handlers = myself.event_handlers.event_handlers.borrow_mut();
                    handlers.id(callback) as u16
                };
                myself.channel.add_listener(id, E::ID, handler_id);
            }
            add_delegated_event_listener(event_name, E::ID as usize, listeners);
        } else {
            myself.queued_listeners.push((id, event_name, callback));
        }
    }

    fn flush(&mut self) {
        let mut myself = self.0.borrow_mut();
        myself.channel.flush();

        for (id, event_name, callback) in myself.queued_listeners.drain(..) {
            let cb = Closure::new(callback);
            let cb_fn: &Function = cb.as_ref().unchecked_ref();
            let node = get_node(id);
            node.add_event_listener_with_callback(event_name, cb_fn)
                .unwrap();
            cb.forget();
        }
    }
}

#[sledgehammer_bindgen::bindgen]
mod js {
    const JS: &str = r#"const nodes = [document.getElementById("main")];
    export function get_node(id){
        return nodes[id];
    }
    export function get_handler_id(id, event_id){
        return nodes[id].getAttribute("data"+event_id);
    }"#;

    extern "C" {
        #[wasm_bindgen]
        fn get_node(id: u32) -> web_sys::Node;
        #[wasm_bindgen]
        fn get_handler_id(id: u32, event_id: usize) -> Option<u32>;
    }

    fn create_element(id: u32, name: &'static str<u8>) {
        r#"nodes[$id$]=document.createElement($name$);"#
    }

    fn create_element_ns(id: u32, name: &'static str<u8>, ns: &'static str<u8>) {
        "nodes[$id$]=document.createElementNS($ns$,$name$);"
    }

    fn create_text(id: u32, text: &str) {
        "nodes[$id$]=document.createTextNode($text$);"
    }

    fn set_style(id: u32, name: &'static str<u8>, val: &str) {
        "nodes[$id$].style[$name$]=$val$;"
    }

    fn set_attribute(id: u32, name: &'static str<u8>, val: &str) {
        "nodes[$id$].setAttribute($name$,$val$);"
    }

    fn remove_attribute(id: u32, name: &'static str<u8>) {
        "nodes[$id$].removeAttribute($name$);"
    }

    fn append_child(id: u32, id2: u32) {
        "nodes[$id$].appendChild(nodes[$id2$]);"
    }

    fn set_text(id: u32, text: &str) {
        "nodes[$id$].textContent=$text$;"
    }

    fn remove(id: u32) {
        "nodes[$id$].remove();"
    }

    fn replace(id: u32, id2: u32) {
        "nodes[$id$].replaceWith(nodes[$id2$]);"
    }

    fn clone(id: u32, id2: u32) {
        "nodes[$id2$]=nodes[$id$].cloneNode(true);"
    }

    fn first_child(id: u32) {
        r#"nodes[id]=nodes[id].firstChild;"#
    }

    fn next_sibling(id: u32) {
        "nodes[id]=nodes[id].nextSibling;"
    }

    fn copy(id: u32, id2: u32) {
        "nodes[$id2$]=nodes[$id$];"
    }

    fn add_listener(id: u32, event_id: u16, handler_id: u16) {
        r#"nodes[$id$].setAttribute("data-"+($event_id$), $handler_id$);"#
    }
}

// A bitset of events that have been registered globally
static EVENT_STATUS: [AtomicU64; EVENT_COUNT / 64] = [AtomicU64::new(0), AtomicU64::new(0)];

fn add_event(event_id: usize) {
    let index = event_id / 64;
    let offset = event_id % 64;
    let encoded = 1 << offset;
    EVENT_STATUS[index].fetch_or(encoded, std::sync::atomic::Ordering::SeqCst);
}

fn get_event(event_id: usize) -> bool {
    let index = event_id / 64;
    let offset = event_id % 64;
    let encoded = 1 << offset;
    EVENT_STATUS[index].load(std::sync::atomic::Ordering::SeqCst) & encoded != 0
}

fn add_delegated_event_listener(
    event_name: &'static str,
    event_id: usize,
    listeners: SharedListeners,
) {
    if !get_event(event_id) {
        let handler = move |ev: web_sys::Event| {
            let target = ev.target();
            let node = ev.composed_path().get(0);
            let node = if node.is_truthy() {
                node
            } else {
                JsValue::from(target)
            };
            let mut node: web_sys::Element = node.unchecked_into();

            while !node.is_null() {
                // navigate up tree
                if let Some(maybe_handler) = node.get_attribute(&format!("data-{event_id}")) {
                    if let Ok(handler_id) = maybe_handler.parse::<u32>() {
                        let mut handlers = listeners.event_handlers.borrow_mut();
                        let handler = handlers.get_mut(handler_id).expect("handler not found");
                        handler(ev.clone());
                    }
                    if ev.cancel_bubble() {
                        return;
                    }
                }
                if let Some(parent) = node.parent_node() {
                    if let Ok(parent) = parent.dyn_into::<web_sys::Element>() {
                        node = parent;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        };

        let handler = Box::new(handler) as Box<dyn FnMut(web_sys::Event)>;
        let handler = Closure::wrap(handler).into_js_value();
        _ = get_node(0).add_event_listener_with_callback(event_name, handler.unchecked_ref());

        add_event(event_id);
    }
}

#[derive(Default, Clone)]
struct SharedListeners {
    event_handlers: Rc<RefCell<IdSlab<Box<dyn FnMut(web_sys::Event)>>>>,
}
