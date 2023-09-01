use super::{
    Block, BlockSourceMap, Expr, ExprId, ExprPtr, Instruction, InstructionId, InstructionPtr,
    Literal,
};
use crate::db::file::File;
use crate::db::Db;
use crate::syntax::{ast, AstToken};

use crate::db::diagnostics::Diagnostics;
use crate::syntax::ast::AstNodeExt;
use la_arena::Arena;
use rustc_hash::FxHashMap;

pub struct BlockCollector<'a> {
    db: &'a dyn Db,
    file: File,
    exprs: Arena<Expr>,
    instructions: Arena<Instruction>,
    exprs_source_map: FxHashMap<ExprId, ExprPtr>,
    instructions_source_map: FxHashMap<InstructionId, InstructionPtr>,
    // TODO: store info on local register aliases
}

impl<'a> BlockCollector<'a> {
    pub fn new(db: &'a dyn Db, file: File) -> Self {
        Self {
            db,
            file,
            exprs: Arena::default(),
            instructions: Arena::default(),
            exprs_source_map: FxHashMap::default(),
            instructions_source_map: FxHashMap::default(),
        }
    }

    fn alloc_expr(&mut self, expr: Expr, ptr: ExprPtr) -> ExprId {
        let expr_id = self.exprs.alloc(expr);
        self.exprs_source_map.insert(expr_id, ptr);

        expr_id
    }
    // FIXME: missing exprs don't have ptr, that's wrong and should be fixed somehow.
    fn missing_expr(&mut self) -> ExprId {
        self.exprs.alloc(Expr::Missing)
    }

    fn alloc_instruction(&mut self, instr: Instruction, ptr: InstructionPtr) -> InstructionId {
        let instr_id = self.instructions.alloc(instr);
        self.instructions_source_map.insert(instr_id, ptr);

        instr_id
    }

    fn collect_int_number(&mut self, literal: ast::IntNumber) -> Option<i32> {
        match literal.value() {
            Ok(v) => Some(v),
            Err(diag) => {
                Diagnostics::emit(self.db, self.file, diag);
                None
            }
        }
    }

    fn collect_literal(&mut self, literal: ast::LiteralKind) -> Literal {
        match literal {
            ast::LiteralKind::String(v) => match v.value() {
                Ok(v) => Literal::String(v.into()),
                Err(diag) => {
                    Diagnostics::emit(self.db, self.file, diag);
                    Literal::String("".into())
                }
            },
            ast::LiteralKind::IntNumber(v) => {
                Literal::IntNumber(self.collect_int_number(v).unwrap_or(1))
            }
            ast::LiteralKind::FloatNumber(_v) => todo!("float number"),
        }
    }

    fn collect_expr(&mut self, expr: ast::Expr) -> ExprId {
        let ptr = expr.ptr();
        match expr {
            ast::Expr::Literal(e) => {
                let literal = self.collect_literal(e.kind());
                self.alloc_expr(Expr::Literal(literal), ptr)
            }
            ast::Expr::NameRefExpr(e) => {
                self.alloc_expr(Expr::NameRef(e.ident().unwrap().text().into()), ptr)
            }
            ast::Expr::RegisterRefExpr(e) => self.alloc_expr(Expr::RegisterRef(e.value()), ptr),
            ast::Expr::ParenExpr(e) => self.collect_expr_opt(e.expr()), // TODO: handle reverse source map
            ast::Expr::ArrayExpr(e) => {
                let mut values = Vec::new();
                for value in e.values() {
                    values.push(self.collect_expr(value))
                }
                self.alloc_expr(Expr::Array(values.into_boxed_slice()), ptr)
            }
            ast::Expr::MappingExpr(e) => {
                let mut arms = Vec::new();
                for arm in e.arms() {
                    let key = arm.key().and_then(|v| self.collect_int_number(v));
                    let body = self.collect_expr_opt(arm.body());
                    arms.push((key, body));
                }

                self.alloc_expr(Expr::Mapping(arms.into_boxed_slice()), ptr)
            }
            ast::Expr::BinExpr(e) => {
                let op = e.op_kind();
                let lhs = self.collect_expr_opt(e.lhs());
                let rhs = self.collect_expr_opt(e.rhs());
                self.alloc_expr(Expr::BinaryOp { lhs, rhs, op }, ptr)
            }
            ast::Expr::PrefixExpr(e) => {
                let inner_expr = self.collect_expr_opt(e.expr());
                if let Some(op) = e.op_kind() {
                    self.alloc_expr(
                        Expr::UnaryOp {
                            expr: inner_expr,
                            op,
                        },
                        ptr,
                    )
                } else {
                    self.missing_expr()
                }
            }
            ast::Expr::CallExpr(_) => todo!(),
        }
    }

    fn collect_expr_opt(&mut self, expr: Option<ast::Expr>) -> ExprId {
        if let Some(expr) = expr {
            self.collect_expr(expr)
        } else {
            self.missing_expr()
        }
    }

    pub fn collect_instruction(&mut self, instr: ast::Instruction) -> InstructionId {
        let name = instr.name().and_then(|v| v.value());

        let args = if let Some(args) = instr.args() {
            args.args().map(|expr| self.collect_expr(expr)).collect()
        } else {
            vec![]
        };

        self.alloc_instruction(
            Instruction {
                name,
                args: args.into_boxed_slice(),
            },
            instr.ptr(),
        )
    }

    pub fn collect(self) -> (Block, BlockSourceMap) {
        (
            Block {
                exprs: self.exprs,
                instructions: self.instructions,
            },
            BlockSourceMap {
                exprs_source_map: self.exprs_source_map,
                instructions_source_map: self.instructions_source_map,
            },
        )
    }
}
