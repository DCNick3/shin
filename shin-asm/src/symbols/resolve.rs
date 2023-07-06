//! Resolving algo:i
//!
//! There isn't much similarity between global and function-level scope resolution. As such, algorithms are different
//!
//! 1. Collect function and global labels, assigning a `CodeRef` to them (how?)
//! 2. Find all `def` items, storing ptrs to bodies in them.
//! 3. Start resolving the bodies using a DFS algorithm (follow references, fail if recursion is detected)

use either::Either;
use internment::{Arena, ArenaIntern};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;

use crate::syntax::AstToken;
use crate::{
    file_db::FileDb,
    file_db::InFile,
    symbols::CodeRef,
    syntax::{ast, ptr::AstPtr},
};

type Str<'a> = ArenaIntern<'a, str>;
type StrArena = Arena<str>;

enum ResolvingSymbolValue {
    CodeLocation(InFile<CodeRef>),
    Unresolved(InFile<AstPtr<ast::Expression>>),
    Resolved(i32),
    Conflict,
}

#[derive(Default)]
struct GlobalResolveContext<'a> {
    symbols: FxHashMap<Str<'a>, ResolvingSymbolValue>,
}

impl<'a> GlobalResolveContext<'a> {
    fn insert_code_location(&mut self, name: Str<'a>, code_ref: InFile<CodeRef>) {
        match self.symbols.entry(name) {
            Entry::Occupied(mut o) => {
                // TODO: emit diagnostic
                o.insert(ResolvingSymbolValue::Conflict);
            }
            Entry::Vacant(v) => {
                v.insert(ResolvingSymbolValue::CodeLocation(code_ref));
            }
        }
    }

    fn insert_unresolved(&mut self, name: Str<'a>, value_ptr: InFile<AstPtr<ast::Expression>>) {
        match self.symbols.entry(name) {
            Entry::Occupied(mut o) => {
                // TODO: emit diagnostic
                o.insert(ResolvingSymbolValue::Conflict);
            }
            Entry::Vacant(v) => {
                v.insert(ResolvingSymbolValue::Unresolved(value_ptr));
            }
        }
    }
}

pub fn resolve(db: &FileDb) {
    let arena = StrArena::new();

    let mut ctx = GlobalResolveContext::default();

    for file_id in db.files() {
        let syntax = db.parse(file_id).tree();

        for (item_index, item) in syntax.items().enumerate() {
            let item_index = item_index as u32;

            match item {
                ast::Item::InstructionsBlock(block) => {
                    for (label_index, label) in block.labels().enumerate() {
                        let label_index: u16 = label_index.try_into().expect("too many labels");

                        if let Some(token) = label.name() {
                            let name = arena.intern(token.text());
                            let code_ref = InFile::new(
                                file_id,
                                CodeRef::Label {
                                    item_index,
                                    label_index,
                                },
                            );

                            ctx.insert_code_location(name, code_ref);
                        }
                    }
                }
                ast::Item::FunctionDefinition(func) => {
                    // here we ignore labels inside the function, as they are not available in the global scope
                    if let Some(name) = func.name() {
                        if let Some(token) = name.token() {
                            let name = arena.intern(token.text());
                            let code_ref = InFile::new(file_id, CodeRef::Function { item_index });

                            ctx.insert_code_location(name, code_ref);
                        }
                    }
                }
                ast::Item::AliasDefinition(alias) => {
                    if let (Some(name), Some(value)) = (alias.name(), alias.value()) {
                        match name {
                            Either::Left(name_def) => {
                                if let Some(token) = name_def.token() {
                                    let name = arena.intern(token.text());
                                    let value_ptr = InFile::new(file_id, AstPtr::new(&value));

                                    ctx.insert_unresolved(name, value_ptr)
                                }
                            }
                            Either::Right(_reg_def) => {
                                // TODO: handle register definitions
                            }
                        }
                    }
                }
            }
        }
    }

    todo!()
}

#[test]
fn test_resolve() {
    let db = FileDb::single_file(
        "test.sal".to_string(),
        r#"

def c = b + 3
def a = 1
def b = a + 2

def x1 = x2
def x2 = x1

function KEK()
LABEL:
    EXIT 0,0
endfun

LABEL:
    EXIT 1,1
    "#
        .to_string(),
    );

    resolve(&db);
}
