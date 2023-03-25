use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    ptr::NonNull,
};

#[derive(Debug)]
pub(crate) struct NodeData {
    pub(crate) ptr: NonNull<()>,
    pub(crate) drop: unsafe fn(*mut ()),
}

#[derive(Debug)]
// Once created, can never be deallocated.
pub(crate) struct Node {
    data: RefCell<Option<NodeData>>,
    next: Cell<Option<&'static Node>>,
    generation: Cell<usize>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct NodeRef {
    node: &'static Node,
    generation: usize,
}

impl NodeRef {
    fn alive(&self) -> bool {
        self.generation == self.node.generation.get()
    }

    /// Safety: The caller must ensure that the type `T` is correct.
    pub(crate) unsafe fn borrow<T>(&self) -> Ref<T> {
        assert!(self.alive());
        let borrow = self.node.data.borrow();
        Ref::map(borrow, |data| unsafe {
            &*(data.as_ref().unwrap().ptr.as_ptr() as *mut T)
        })
    }

    /// Safety: The caller must ensure that the type `T` is correct.
    pub(crate) unsafe fn borrow_mut<T>(&self) -> RefMut<T> {
        assert!(self.alive());
        let borrow = self.node.data.borrow_mut();
        RefMut::map(borrow, |data| unsafe {
            &mut *(data.as_ref().unwrap().ptr.as_ptr() as *mut T)
        })
    }
}

#[derive(Default)]
pub(crate) struct Queue {
    head: Cell<Option<&'static Node>>,
}

impl Queue {
    pub(crate) fn insert(&self, data: NodeData) -> NodeRef {
        self.insert_with(|_| data)
    }

    pub(crate) fn insert_with(&self, f: impl FnOnce(NodeRef) -> NodeData) -> NodeRef {
        match self.head.get() {
            Some(head) => {
                let node = NodeRef {
                    node: head,
                    generation: head.generation.get(),
                };

                // update the head of the list
                self.head.set(head.next.get());

                // create the node with a reference to the data that will panic if used
                let data = f(node);

                // Insert the data into the node. It is now possible to access the data
                *head.data.borrow_mut() = Some(data);

                node
            }
            None => {
                let node = Node {
                    data: RefCell::new(None),
                    next: Cell::new(None),
                    generation: Cell::new(0),
                };
                let node = Box::leak(Box::new(node));
                let node = NodeRef {
                    node,
                    generation: 0,
                };
                let data = f(node);
                *node.node.data.borrow_mut() = Some(data);
                node
            }
        }
    }

    pub(crate) unsafe fn remove(&self, node: NodeRef) {
        // invalidate the pointer by incrementing the generation
        node.node.generation.set(node.generation + 1);

        // drop the data
        let mut data = node.node.data.borrow_mut();
        let data = data.take().unwrap();
        (data.drop)(data.ptr.as_ptr());

        // reinsert the node at the head of the list
        node.node.next.set(self.head.get());
        self.head.set(Some(node.node));
    }
}
