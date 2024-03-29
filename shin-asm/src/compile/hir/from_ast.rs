use la_arena::Arena;
use rustc_hash::FxHashMap;
use shin_core::rational::Rational;
use text_size::TextRange;

use super::{
    BlockSourceMap, Expr, ExprId, ExprPtr, HirBlockBody, Instruction, InstructionId,
    InstructionPtr, Literal,
};
use crate::{
    compile::{def_map::Name, diagnostics::Diagnostic, hir::lower::LowerError},
    syntax::{ast, ast::AstNodeExt, AstToken},
};

pub struct HirBlockCollector {
    diagnostics: Vec<Diagnostic<TextRange>>,
    exprs: Arena<Expr>,
    instructions: Arena<Instruction>,
    exprs_source_map: FxHashMap<ExprId, ExprPtr>,
    instructions_source_map: FxHashMap<InstructionId, InstructionPtr>,
    // TODO: store info on local register aliases
}

impl HirBlockCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            exprs: Arena::default(),
            instructions: Arena::default(),
            exprs_source_map: FxHashMap::default(),
            instructions_source_map: FxHashMap::default(),
        }
    }

    fn handle_result<T>(&mut self, result: Result<T, Diagnostic<TextRange>>, fallback: T) -> T {
        match result {
            Ok(v) => v,
            Err(e) => {
                self.diagnostics.push(e);
                fallback
            }
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
        self.handle_result(literal.value().map(Some), None)
    }

    fn collect_rational_number(&mut self, literal: ast::RationalNumber) -> Option<Rational> {
        self.handle_result(literal.value().map(Some), None)
    }

    fn collect_literal(&mut self, literal: ast::LiteralKind) -> Literal {
        match literal {
            ast::LiteralKind::String(v) => self.handle_result(
                v.value().map(|v| Literal::String(v.into())),
                Literal::String("".into()),
            ),
            ast::LiteralKind::IntNumber(v) => {
                Literal::IntNumber(self.collect_int_number(v).unwrap_or(1))
            }
            ast::LiteralKind::RationalNumber(v) => {
                Literal::RationalNumber(self.collect_rational_number(v).unwrap_or(Rational::ONE))
            }
        }
    }

    pub fn collect_expr(&mut self, expr: ast::Expr) -> ExprId {
        let ptr = expr.ptr();
        match expr {
            ast::Expr::Literal(e) => {
                let literal = self.collect_literal(e.kind());
                self.alloc_expr(Expr::Literal(literal), ptr)
            }
            ast::Expr::NameRefExpr(e) => {
                self.alloc_expr(Expr::NameRef(Name(e.ident().unwrap().text().into())), ptr)
            }
            ast::Expr::RegisterRefExpr(e) => {
                let register = match e.value().kind() {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        self.diagnostics.push(e);
                        Err(LowerError)
                    }
                };
                self.alloc_expr(Expr::RegisterRef(register), ptr)
            }
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

    pub fn collect(self) -> (HirBlockBody, BlockSourceMap, Vec<Diagnostic<TextRange>>) {
        (
            HirBlockBody {
                exprs: self.exprs,
                instructions: self.instructions,
            },
            BlockSourceMap {
                expressions_source_map: self.exprs_source_map,
                instructions_source_map: self.instructions_source_map,
            },
            self.diagnostics,
        )
    }
}
