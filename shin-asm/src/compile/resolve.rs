use crate::{
    compile::Db,
    syntax::ast::{self},
};
use shin_core::format::scenario::instruction_elements::Register;

pub struct ResolveContext<'a> {
    _db: &'a dyn Db,
}

impl<'a> ResolveContext<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self { _db: db }
    }

    pub fn resolve_register(&self, register: &ast::RegisterIdentKind) -> Option<Register> {
        match register {
            &ast::RegisterIdentKind::Register(register) => Some(register),
            ast::RegisterIdentKind::Alias(_) => todo!(),
        }
    }

    // pub fn resolve_definition(&self, _name: &Name) -> Option<DefRef> {
    //     todo!()
    // }
}
