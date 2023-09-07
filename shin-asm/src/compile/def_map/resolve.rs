use crate::compile::{BlockIdWithFile, DefMap, DefRef, MakeWithFile, WithFile};
use crate::syntax::ast;
use crate::{
    compile::{
        def_map::{Name, RegisterName},
        Db,
    },
    elements::Register,
    syntax::ast::visit::ItemIndex,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstexprValue(Option<i32>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResolvedDef {
    Undefined,
    Block(BlockIdWithFile),
    Define(ConstexprValue),
}

#[salsa::tracked]
pub struct ResolvedDefMap {
    #[return_ref]
    def_map: DefMap,
}

#[salsa::tracked]
impl ResolvedDefMap {
    #[salsa::tracked]
    pub fn get_value(self, db: &dyn Db, name: Name) -> ResolvedDef {
        match self.def_map(db).get_item(&name) {
            None => ResolvedDef::Undefined,
            Some(WithFile {
                file,
                value: DefRef::Block(block),
            }) => ResolvedDef::Block(block.in_file(file)),
            Some(WithFile {
                file,
                value: DefRef::Define(define_item_index),
            }) => {
                let syntax = file.parse(db);
                let define_item_index: u32 = define_item_index.into();
                let ast::Item::AliasDefinition(def) =
                    syntax.items().nth(define_item_index as usize).unwrap()
                else {
                    unreachable!()
                };

                let dummy = ResolvedDef::Define(ConstexprValue(None));

                let Some(value) = def.value() else {
                    return dummy;
                };

                // crate::compile::hir::

                todo!()
            }
        }
    }

    #[salsa::tracked]
    pub fn get_register(
        self,
        db: &dyn Db,
        function_scope: Option<ItemIndex>,
        name: RegisterName,
    ) -> Option<Register> {
        todo!()
    }
}
