use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::copy_ll::{NodeData, NodeRef, Queue};

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RuntimeId;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RuntimeId(usize);

#[cfg(feature = "ssr")]
thread_local! {
    static RUNTIMES: RefCell<slotmap::SlotMap<RuntimeId, Runtime>> = RefCell::new(SlotMap::default());
}

#[cfg(not(feature = "ssr"))]
thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

impl RuntimeId {
    pub fn create() -> Self {
        #[cfg(feature = "ssr")]
        return RUNTIMES.with(|runtimes| {
            let mut runtimes = runtimes.borrow_mut();
            runtimes.insert(Runtime::new())
        });
        #[cfg(not(feature = "ssr"))]
        return RuntimeId;
    }
}

pub(crate) fn with_rt<O>(runtime_id: RuntimeId, f: impl FnOnce(&Runtime) -> O) -> O {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = runtime_id;
        RUNTIME.with(f)
    }
    #[cfg(feature = "ssr")]
    RUNTIMES.with(|runtimes| {
        let runtimes = runtimes.borrow();
        let runtime = runtimes
            .get(runtime_id)
            .expect("tried to get a runtime that was dropped");
        f(runtime)
    })
}

/// Provide the runtime for signals
///
/// This will reuse dead runtimes
pub fn claim_rt() -> RuntimeId {
    #[cfg(not(feature = "ssr"))]
    return RuntimeId;
    #[cfg(feature = "ssr")]
    RUNTIMES.with(|runtimes| runtimes.borrow_mut().insert(Runtime::new()))
}

/// Removes the runtime from the thread local storage
/// This will drop all signals and effects
pub fn drop_rt(runtime_id: RuntimeId) {
    #[cfg(not(feature = "ssr"))]
    let _ = runtime_id;
    #[cfg(feature = "ssr")]
    RUNTIMES.with(|runtimes| {
        runtimes.borrow_mut().remove(runtime_id);
    });
}

pub struct Runtime {
    pub(crate) states: Queue,
}

impl Runtime {
    fn new() -> Self {
        Self {
            states: Queue::default(),
        }
    }
}

#[macro_export]
macro_rules! hyristic {
    () => {
        struct Hyristics;
        static mut GUESS: usize = 0;

        impl $crate::copy::ScopeHyristics for Hyristics {
            fn guess_allocation() -> usize {
                unsafe { GUESS }
            }

            fn update_guess(new: usize) {
                unsafe { GUESS = new }
            }
        }
    };
}

#[macro_export]
macro_rules! hyristic2 {
    () => {
        struct Hyristics2;
        static mut GUESS2: usize = 0;

        impl $crate::copy::ScopeHyristicsOwned for Hyristics2 {
            fn guess_owned() -> usize {
                unsafe { GUESS2 }
            }

            fn update_owned(new: usize) {
                unsafe { GUESS2 = new }
            }
        }
    };
}

#[cfg(feature = "bump")]
#[macro_export]
macro_rules! scope {
    ($runtime:expr) => {{
        $crate::hyristic!();
        $crate::hyristic2!();
        $crate::copy::Scope::new::<Hyristics, Hyristics2>($runtime)
    }};
}

#[cfg(not(feature = "bump"))]
#[macro_export]
macro_rules! scope {
    ($runtime:expr) => {{
        $crate::hyristic2!();
        $crate::copy::Scope::new::<Hyristics2>($runtime)
    }};
}

#[cfg(feature = "bump")]
#[macro_export]
macro_rules! child_scope {
    ($scope:expr, $closure:expr) => {{
        $crate::hyristic!();
        $crate::hyristic2!();
        $scope.child::<Hyristics, Hyristics2, _>($closure)
    }};
}

#[cfg(not(feature = "bump"))]
#[macro_export]
macro_rules! child_scope {
    ($scope:expr, $closure:expr) => {{
        $crate::hyristic2!();
        $scope.child::<Hyristics2, _>($closure)
    }};
}

#[cfg(feature = "bump")]
pub trait ScopeHyristics {
    fn guess_allocation() -> usize;
    fn update_guess(new: usize);
}

pub trait ScopeHyristicsOwned {
    fn guess_owned() -> usize;
    fn update_owned(new: usize);
}

pub struct Scope {
    parent: Option<RuntimeId>,
    children: RefCell<Option<Vec<Scope>>>,
    runtime: RuntimeId,
    owns: RefCell<Vec<NodeRef>>,
    update_owned: fn(usize),
    #[cfg(feature = "bump")]
    update: fn(usize),
    #[cfg(feature = "bump")]
    allocator: bumpalo::Bump,
}

impl Scope {
    #[cfg(feature = "bump")]
    pub fn new<H: ScopeHyristics, H2: ScopeHyristicsOwned>(runtime: RuntimeId) -> Self {
        Self {
            parent: None,
            children: Default::default(),
            runtime,
            owns: RefCell::new(Vec::with_capacity(H2::guess_owned())),
            update_owned: H2::update_owned,
            #[cfg(feature = "bump")]
            update: H::update_guess,
            #[cfg(feature = "bump")]
            allocator: bumpalo::Bump::with_capacity(H::guess_allocation()),
        }
    }

    #[cfg(not(feature = "bump"))]
    pub fn new<H: ScopeHyristicsOwned>(runtime: RuntimeId) -> Self {
        Self {
            parent: None,
            children: Default::default(),
            runtime,
            owns: Default::default(),
            update_owned: H::update_owned,
        }
    }

    #[cfg(feature = "bump")]
    pub fn child<H: ScopeHyristics, H2: ScopeHyristicsOwned, O>(
        &self,
        f: impl FnOnce(&Scope) -> O,
    ) -> O {
        let scope = Self {
            parent: Some(self.runtime),
            children: Default::default(),
            runtime: self.runtime,
            owns: RefCell::new(Vec::with_capacity(H2::guess_owned())),
            update_owned: H2::update_owned,
            update: H::update_guess,
            allocator: bumpalo::Bump::with_capacity(H::guess_allocation()),
        };
        let r = f(&scope);
        (scope.update)(scope.allocator.allocated_bytes());
        (self.update_owned)(scope.owns.borrow().len());
        self.children
            .borrow_mut()
            .get_or_insert(Default::default())
            .push(scope);
        r
    }

    #[cfg(not(feature = "bump"))]
    pub fn child<H: ScopeHyristicsOwned, O>(&self, f: impl FnOnce(&Scope) -> O) -> O {
        let scope = Self {
            parent: Some(self.runtime),
            children: Default::default(),
            runtime: self.runtime,
            owns: RefCell::new(Vec::with_capacity(H::guess_owned())),
            update_owned: H::update_owned,
        };
        let r = f(&scope);
        (self.update_owned)(scope.owns.borrow().len());
        self.children
            .borrow_mut()
            .get_or_insert(Default::default())
            .push(scope);
        r
    }

    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        #[cfg(feature = "bump")]
        let non_null: NonNull<T> = self.allocator.alloc(value).into();
        #[cfg(not(feature = "bump"))]
        let non_null: NonNull<T> =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) };
        let raw = with_rt(self.runtime, |runtime| {
            runtime.states.insert(NodeData {
                ptr: non_null.cast(),
                drop: |value: *mut ()| unsafe {
                    std::ptr::drop_in_place(value as *mut T);
                },
            })
        });
        let signal = State {
            raw,
            phantom: PhantomData,
        };
        self.owns.borrow_mut().push(raw);
        signal
    }

    pub fn state_with<T: 'static>(&self, constructor: impl FnOnce(State<T>) -> T) -> State<T> {
        let key = with_rt(self.runtime, |runtime| {
            runtime.states.insert_with(|raw| {
                let signal = State {
                    raw,
                    phantom: PhantomData,
                };
                let value = constructor(signal);
                #[cfg(feature = "bump")]
                let non_null: NonNull<T> = self.allocator.alloc(value).into();
                #[cfg(not(feature = "bump"))]
                let non_null: NonNull<T> =
                    unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) };

                NodeData {
                    ptr: non_null.cast(),
                    drop: |value: *mut ()| unsafe {
                        std::ptr::drop_in_place(value as *mut T);
                    },
                }
            })
        });
        self.owns.borrow_mut().push(key);
        State {
            raw: key,
            phantom: PhantomData,
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        with_rt(self.runtime, |runtime| {
            for key in self.owns.borrow().iter() {
                unsafe {
                    runtime.states.remove(*key);
                }
            }
        });
        #[cfg(feature = "bump")]
        {
            let new_guess = self.allocator.allocated_bytes();
            (self.update)(new_guess);
        }
        let new_guess = self.owns.borrow().len();
        (self.update_owned)(new_guess);
    }
}

pub struct State<T: ?Sized + 'static> {
    pub(crate) raw: NodeRef,
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<T: Display> Display for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|x| x.fmt(f))
    }
}

impl<T: Debug> Debug for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|x| x.fmt(f))
    }
}

impl<T: 'static> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            phantom: self.phantom,
        }
    }
}

impl<T: 'static> Copy for State<T> {}

impl<T: 'static> State<T> {
    pub fn set(&self, value: T) {
        self.modify(|x| *x = value)
    }

    pub fn with<U: 'static, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        unsafe {
            let r = self.raw.borrow::<T>();
            f(&*r)
        }
    }

    pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
        unsafe {
            let mut r = self.raw.borrow_mut::<T>();
            f(&mut *r)
        }
    }
}

impl<T: 'static + Copy> State<T> {
    pub fn get(&self) -> T {
        self.with(|x| *x)
    }
}
