pub trait Renderer {
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

    fn flush(&mut self) {}
}
