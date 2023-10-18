mod collect;
mod items;
mod registers;

pub use items::ResolvedItems;
pub use registers::{LocalRegisters, ResolvedGlobalRegisters};
use std::borrow::Cow;

use crate::compile::Program;
use crate::compile::{BlockIdWithFile, Db};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::fmt::Display;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Name(pub SmolStr);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct RegisterName(pub SmolStr);

impl Display for RegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BlockName {
    GlobalBlock(Option<Name>),
    Function(Option<Name>),
    LocalBlock(Option<Name>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DefMap {
    items: ResolvedItems,
    global_registers: ResolvedGlobalRegisters,
    local_registers: LocalRegisters,
    block_names: FxHashMap<BlockIdWithFile, BlockName>,
}

impl DefMap {
    // pub fn get_item(&self, name: &Name) -> Option<FileDefRef> {
    //     self.items.get(name).copied()
    // }
}

impl DefMap {
    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write as _;

        let mut output = String::new();

        let mut items = self.items.iter().collect::<Vec<_>>();
        items.sort_by_key(|&(name, _)| name);

        writeln!(output, "items:").unwrap();
        for (name, value) in items {
            writeln!(output, "  {}: {:?}", name, value).unwrap();
        }

        let mut global_registers = self.global_registers.iter().collect::<Vec<_>>();
        global_registers.sort_by_key(|(name, _)| *name);
        let mut local_registers = self.local_registers.iter().collect::<Vec<_>>();
        local_registers.sort_by_key(|(&index, _)| index);

        writeln!(output, "registers:").unwrap();
        writeln!(output, "  global:").unwrap();
        for (name, value) in global_registers {
            writeln!(
                output,
                "    {}: {}",
                name,
                value.map_or(Cow::from("[ERROR]"), |r| format!("{:?}", r).into())
            )
            .unwrap();
        }
        writeln!(output, "  local:").unwrap();
        for (item_index, registers) in local_registers {
            writeln!(output, "    item {}: ", item_index).unwrap();
            for (name, value) in registers {
                writeln!(output, "      {}: {:?}", name, value).unwrap();
            }
        }

        let mut block_names = self.block_names.iter().collect::<Vec<_>>();
        block_names.sort();

        writeln!(output, "block names:").unwrap();
        for (block_id, name) in block_names {
            let file_name = block_id.file.path(db);
            let block_id = &block_id.value;

            writeln!(output, "  {:?} @ {}: {:?}", block_id, file_name, name).unwrap();
        }

        output
    }
}

#[salsa::tracked]
pub fn build_def_map(db: &dyn Db, program: Program) -> DefMap {
    let items = items::collect_item_defs(db, program);
    let items = items::resolve_item_defs(db, &items);

    let local_registers = registers::collect_local_registers(db, program);
    let global_registers = registers::collect_global_registers(db, program);
    let global_registers = registers::resolve_global_registers(db, &global_registers);
    let block_names = collect::collect_block_names(db, program);

    DefMap {
        items,
        local_registers,
        global_registers,
        block_names,
    }
}

#[cfg(test)]
mod tests {
    use super::build_def_map;
    use crate::compile::diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator};
    use crate::compile::{db::Database, DefMap, File, Program};
    use expect_test::expect;

    fn parse_def_map(code: &str) -> (Database, DefMap, Option<String>) {
        let db = Database::default();
        let file = File::new(&db, "test.sal".to_string(), code.to_string());
        let program = Program::new(&db, vec![file]);
        let def_map = build_def_map(&db, program);

        let hir_errors = build_def_map::accumulated::<HirDiagnosticAccumulator>(&db, program);
        let source_errors = build_def_map::accumulated::<SourceDiagnosticAccumulator>(&db, program);

        let errors = (!source_errors.is_empty() || !hir_errors.is_empty()).then(|| {
            format!(
                "building def map produced errors:\n\
                source-level: {source_errors:?}\n\
                hir-level: {hir_errors:?}"
            )
        });

        (db, def_map, errors)
    }

    #[test]
    fn check_map_dump() {
        let (db, def_map, errors) = parse_def_map(
            r#"
def ABIBA = 3 + 3
def BEBA = ABIBA * 2
def $_aboba = $v17
def $keka = $_aboba

function KEKA($a0, $hello, $keka)
    add $v1, 2, 2
ABOBA:
    add $v1, 3, 3
endfun

    add $v2, 2, 2
LABEL1:
    sub $v2, 2, 2
    j LABEL1
LABEL2:
        "#,
        );

        assert!(errors.is_none());

        expect![[r#"
            items:
              ABIBA: Value(6)
              BEBA: Value(12)
              KEKA: Block(WithFile { value: BlockId { item_index: 4, block_index: None }, file: File(Id { value: 1 }) })
              LABEL1: Block(WithFile { value: BlockId { item_index: 5, block_index: Some(1) }, file: File(Id { value: 1 }) })
              LABEL2: Block(WithFile { value: BlockId { item_index: 5, block_index: Some(2) }, file: File(Id { value: 1 }) })
            registers:
              global:
                _aboba: $v17
                keka: $v17
              local:
                item #4: 
                  hello: $a1
                  keka: $a2
            block names:
              BlockId { item_index: 4, block_index: None } @ test.sal: Function(Some(Name("KEKA")))
              BlockId { item_index: 4, block_index: Some(0) } @ test.sal: LocalBlock(None)
              BlockId { item_index: 4, block_index: Some(1) } @ test.sal: LocalBlock(Some(Name("ABOBA")))
              BlockId { item_index: 5, block_index: Some(0) } @ test.sal: GlobalBlock(None)
              BlockId { item_index: 5, block_index: Some(1) } @ test.sal: GlobalBlock(Some(Name("LABEL1")))
              BlockId { item_index: 5, block_index: Some(2) } @ test.sal: GlobalBlock(Some(Name("LABEL2")))
        "#]].assert_eq(&def_map.debug_dump(&db));
    }

    #[test]
    fn register_loop() {
        let (db, def_map, errors) = parse_def_map(
            r#"
def $a = $b
def $b = $a
        "#,
        );

        expect![[r#"
            building def map produced errors:
            source-level: [Diagnostic { message: "Encountered a loop while resolving register $b", location: Span(WithFile { value: 10..12, file: File(Id { value: 1 }) }), additional_labels: [] }]
            hir-level: []"#]]
            .assert_eq(errors.as_deref().unwrap());

        expect![[r#"
            items:
            registers:
              global:
                a: [ERROR]
                b: [ERROR]
              local:
            block names:
        "#]]
        .assert_eq(&def_map.debug_dump(&db));
    }

    #[test]
    fn constexpr_overflow() {
        let (db, def_map, errors) = parse_def_map(
            r#"
def A = 65536 * 65536
        "#,
        );

        expect![[r#"
            building def map produced errors:
            source-level: []
            hir-level: []"#]]
        .assert_eq(errors.as_deref().unwrap());

        expect![[r#"
            items:
              A: Value(ConstexprValue(<dummy>))
            registers:
              global:
              local:
            block names:
        "#]]
        .assert_eq(&def_map.debug_dump(&db));
    }
}
