use crate::db::in_file::InFile;
use crate::db::symbols::DefRefId;
use crate::syntax::ast;
use crate::syntax::ptr::AstPtr;
use smol_str::SmolStr;

// // #[salsa::tracked]
// pub enum Item {
//     AliasDefinition,
//     FunctionDefinition,
//     InstructionsBlock,
// }
//
// #[salsa::tracked]
// pub struct AliasDefinition {
//     pub name: SmolStr,
//     pub ptr: InFile<AstPtr<ast::AliasDefinition>>,
// }
//
// // #[salsa::tracked]
// pub struct FunctionDefinition {
//     pub name: SmolStr,
//     pub code_ref: CodeRefId,
//     pub ptr: InFile<AstPtr<ast::FunctionDefinition>>,
// }
