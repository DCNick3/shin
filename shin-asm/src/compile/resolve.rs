use shin_core::format::scenario::instruction_elements::Register;

use crate::{
    compile::{
        def_map::{DefValue, Name, RegisterName, ResolveKind},
        Db, DefMap,
    },
    syntax::ast::{self},
};

#[derive(Debug, Copy, Clone)]
enum ResolveContextInner {
    Empty,
    Real {
        def_map: DefMap,
        resolve_kind: ResolveKind,
    },
}

pub struct ResolveContext<'db> {
    db: &'db dyn Db,
    inner: ResolveContextInner,
}

impl<'a> ResolveContext<'a> {
    pub fn new_empty(db: &'a dyn Db) -> Self {
        Self {
            db,
            inner: ResolveContextInner::Empty,
        }
    }

    pub fn new(db: &'a dyn Db, def_map: DefMap, resolve_kind: ResolveKind) -> Self {
        Self {
            db,
            inner: ResolveContextInner::Real {
                def_map,
                resolve_kind,
            },
        }
    }

    pub fn resolve_register(&self, register: &ast::RegisterIdentKind) -> Option<Register> {
        match register {
            &ast::RegisterIdentKind::Register(register) => Some(register),
            ast::RegisterIdentKind::Alias(name) => match self.inner {
                ResolveContextInner::Empty => None,
                ResolveContextInner::Real {
                    def_map,
                    resolve_kind,
                } => def_map.resolve_register(self.db, RegisterName(name.clone()), resolve_kind),
            },
        }
    }

    pub fn resolve_item(&self, name: &Name) -> Option<DefValue> {
        match self.inner {
            ResolveContextInner::Empty => None,
            ResolveContextInner::Real {
                def_map,
                resolve_kind: _,
            } => def_map.resolve_item(self.db, name.clone()),
        }
    }
}
