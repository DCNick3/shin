//! Contains breakpoint functionality for the VM

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Weak,
    },
};

use crate::format::scenario::instruction_elements::CodeAddress;

pub(crate) struct Breakpoint {
    hit_count: AtomicU32,
}

impl Breakpoint {
    pub fn new() -> Self {
        Self {
            hit_count: AtomicU32::new(0),
        }
    }
}

pub(crate) struct CodeBreakpointSet(HashMap<CodeAddress, Weak<Breakpoint>>);

impl CodeBreakpointSet {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    // TODO: maybe capture some more stuff that may useful for debugging, like register values or smth?
    // maybe oven provide an observer interface?
    // :shrug:
    // For now I only care about how many times the VM hit the address
    pub fn visit_address(&mut self, addr: CodeAddress) {
        match self.0.entry(addr) {
            Entry::Occupied(e) => {
                match e.get().upgrade() {
                    None => {
                        // the weak ref is dead, remove the breakpoint
                        e.remove();
                    }
                    Some(b) => {
                        b.hit_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            Entry::Vacant(_) => {}
        }
    }

    pub fn add_breakpoint(&mut self, address: CodeAddress) -> BreakpointHandle {
        match self.0.entry(address) {
            Entry::Occupied(mut e) => match e.get().upgrade() {
                None => {
                    let result = Arc::new(Breakpoint::new());
                    e.insert(Arc::downgrade(&result));
                    BreakpointHandle(result)
                }
                Some(v) => BreakpointHandle(v),
            },
            Entry::Vacant(e) => {
                let result = Arc::new(Breakpoint::new());
                e.insert(Arc::downgrade(&result));
                BreakpointHandle(result)
            }
        }
    }
}

/// A handle to a breakpoint
///
/// It allows to check how many times the breakpoint was hit
///
/// When it is dropped, the breakpoint is removed from the VM (lazily)
#[derive(Clone)]
pub struct BreakpointHandle(Arc<Breakpoint>);

impl BreakpointHandle {
    pub fn hit_count(&self) -> u32 {
        self.0.hit_count.load(Ordering::SeqCst)
    }
}

/// Combines handle to the breakpoint with a counter, allowing to check whether the BP was hit between [BreakpointObserver::update] calls
#[derive(Clone)]
pub struct BreakpointObserver {
    handle: BreakpointHandle,
    old_count: u32,
}

impl BreakpointObserver {
    pub fn new(handle: BreakpointHandle) -> Self {
        Self {
            old_count: handle.hit_count(),
            handle,
        }
    }

    /// Checks whether the breakpoint was hit after the last call to update (or creation)
    pub fn update(&mut self) -> bool {
        let new_count = self.handle.hit_count();
        let was_hit = self.old_count != new_count;
        self.old_count = new_count;
        was_hit
    }
}

impl From<BreakpointHandle> for BreakpointObserver {
    fn from(handle: BreakpointHandle) -> Self {
        Self::new(handle)
    }
}
