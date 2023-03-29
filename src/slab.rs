pub struct IdSlab<T> {
    last_node_id: u32,
    recyled_nodes: Vec<u32>,
    data: Vec<Option<T>>,
}

impl<T> Default for IdSlab<T> {
    fn default() -> Self {
        Self {
            last_node_id: 0,
            recyled_nodes: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<T> IdSlab<T> {
    pub fn id(&mut self, data: T) -> u32 {
        match self.recyled_nodes.pop() {
            Some(id) => {
                self.data[id as usize] = Some(data);
                id
            }
            None => {
                let current = self.last_node_id;
                self.last_node_id += 1;
                self.data.push(Some(data));
                current
            }
        }
    }

    pub fn recycle(&mut self, id: u32) {
        self.recyled_nodes.push(id);
        self.data[id as usize] = None;
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.data[id as usize].as_mut()
    }
}
