use crate::renderer::Renderer;

pub struct WebRenderer {
    channel: Channel,
    last_node_id: u32,
    recyled_nodes: Vec<u32>,
}

impl Default for WebRenderer {
    fn default() -> Self {
        Self {
            channel: Channel::default(),
            last_node_id: 1,
            recyled_nodes: Vec::new(),
        }
    }
}

impl Renderer for WebRenderer {
    fn node(&mut self) -> u32 {
        match self.recyled_nodes.pop() {
            Some(id) => id,
            None => {
                let current = self.last_node_id;
                self.last_node_id += 1;
                current
            }
        }
    }

    fn append_all(&mut self, parent: u32, children: impl IntoIterator<Item = u32>) {
        for child in children.into_iter() {
            self.append_child(parent, child);
        }
    }

    fn set_attribute(&mut self, id: u32, name: &'static str, value: &str) {
        self.channel.set_attribute(id, name, value);
    }

    fn set_style(&mut self, id: u32, name: &'static str, value: &str) {
        self.channel.set_style(id, name, value);
    }

    fn create_element(&mut self, id: u32, tag: &'static str) {
        self.channel.create_element(id, tag);
    }

    fn create_text(&mut self, id: u32, text: &str) {
        self.channel.create_text(id, text);
    }

    fn set_text(&mut self, id: u32, text: &str) {
        self.channel.set_text(id, text);
    }

    fn append_child(&mut self, parent: u32, child: u32) {
        self.channel.append_child(parent, child);
    }

    fn clone_node(&mut self, id: u32, new_id: u32) {
        self.channel.clone(id, new_id);
    }

    fn copy(&mut self, id: u32, id2: u32) {
        self.channel.copy(id, id2);
    }

    fn first_child(&mut self, id: u32) {
        self.channel.first_child(id);
    }

    fn next_sibling(&mut self, id: u32) {
        self.channel.next_sibling(id);
    }

    fn remove(&mut self, id: u32) {
        self.channel.remove(id);
    }

    fn return_node(&mut self, id: u32) {
        self.recyled_nodes.push(id);
    }

    fn flush(&mut self) {
        self.channel.flush();
    }
}

#[sledgehammer_bindgen::bindgen]
mod js {
    const JS: &str = r#"const nodes = [document.getElementById("main")];
    export function get_node(id){
        return nodes[id];
    }"#;

    extern "C" {
        #[wasm_bindgen]
        fn get_node(id: u32) -> web_sys::Node;
    }

    fn create_element(id: u32, name: &'static str<u8, name_cache>) {
        r#"nodes[$id$]=document.createElement($name$);"#
    }

    fn create_element_ns(
        id: u32,
        name: &'static str<u8, name_cache>,
        ns: &'static str<u8, ns_cache>,
    ) {
        "nodes[$id$]=document.createElementNS($ns$,$name$);"
    }

    fn create_text(id: u32, text: &str) {
        "nodes[$id$]=document.createTextNode($text$);"
    }

    fn set_style(id: u32, name: &'static str<u8, name_cache>, val: &str) {
        "nodes[$id$].style[$name$]=$val$;"
    }

    fn set_attribute(id: u32, name: &'static str<u8, name_cache>, val: &str) {
        "nodes[$id$].setAttribute($name$,$val$);"
    }

    fn remove_attribute(id: u32, name: &'static str<u8, name_cache>) {
        "nodes[$id$].removeAttribute($name$);"
    }

    fn append_child(id: u32, id2: u32) {
        "nodes[$id$].appendChild(nodes[$id2$]);"
    }

    fn insert_before(parent: u32, id: u32, id2: u32) {
        "nodes[$parent$].insertBefore(nodes[$id$],nodes[$id2$]);"
    }

    fn set_text(id: u32, text: &str) {
        "nodes[id].textContent=$text$;"
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
        "nodes[id2]=nodes[id];"
    }
}
