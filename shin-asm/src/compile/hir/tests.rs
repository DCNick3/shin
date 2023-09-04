use crate::compile::{db::Database, hir, Diagnostics, File};
use expect_test::expect_file;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use test_generator::test_resources;

fn lower_block(code: &str) -> Rc<hir::HirBlockBody> {
    use std::fmt::Write as _;

    let db = Database::default();
    let db = &db;
    let file = File::new(db, "test.sal".to_string(), code.to_string());
    let bodies = hir::collect_file_bodies(db, file);

    let diagnostics = hir::collect_file_bodies::accumulated::<Diagnostics>(db, file);
    let mut errors = String::new();
    for diagnostic in Diagnostics::with_source(db, diagnostics) {
        writeln!(errors, "{:?}", diagnostic).unwrap();
    }
    if !errors.is_empty() {
        panic!("lowering produced errors:\n{}", errors);
    }

    let block_ids = bodies.get_block_ids(db);
    assert_eq!(block_ids.len(), 1, "expected exactly one block");
    bodies.get(db, block_ids[0]).unwrap().clone()
}

#[test_resources("test_data/hir/ok/*.sal")]
fn lower_ok(sal: &str) {
    let case = TestCase::from_sal_path(sal);
    let block_body = lower_block(&case.text);
    let block_body = block_body.debug_dump();
    expect_file![case.hir].assert_eq(&block_body);
}

// #[test_resources("test_data/parser/err/*.sal")]
// fn parse_err(sal: &str) {
//     let case = TestCase::from_sal_path(sal);
//     let (actual, errors) = parse(&case.text);
//     assert!(
//         errors,
//         "no errors in an ERR file {}:\n{actual}",
//         case.sal.display()
//     );
//     expect_file![case.sast].assert_eq(&actual)
// }

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct TestCase {
    sal: PathBuf,
    hir: PathBuf,
    text: String,
}

impl TestCase {
    fn from_sal_path(path: &str) -> TestCase {
        let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sal = crate_root_dir.join(path);
        let hir = sal.with_extension("hir");
        let text = fs::read_to_string(&sal).unwrap();
        TestCase { sal, hir, text }
    }
}
