use expect_test::expect_file;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    parser::{shortcuts::StrStep, LexedStr},
    util::panic_context,
};

#[test]
fn parse_ok() {
    for case in TestCase::list("parser/ok") {
        let _guard = panic_context::enter(format!("{:?}", case.sal));
        let (actual, errors) = parse(&case.text);
        assert!(
            !errors,
            "errors in an OK file {}:\n{actual}",
            case.sal.display()
        );
        expect_file![case.sast].assert_eq(&actual);
    }
}

fn parse(text: &str) -> (String, bool) {
    let lexed = LexedStr::new(text);
    let input = lexed.to_input();
    let output = crate::parser::parse(&input);

    let mut buf = String::new();
    let mut errors = Vec::new();
    let mut indent = String::new();
    let mut depth = 0;
    let mut len = 0;
    lexed.intersperse_trivia(&output, &mut |step| match step {
        StrStep::Token { kind, text } => {
            assert!(depth > 0);
            len += text.len();
            writeln!(buf, "{indent}{kind:?} {text:?}").unwrap();
        }
        StrStep::Enter { kind } => {
            assert!(depth > 0 || len == 0);
            depth += 1;
            writeln!(buf, "{indent}{kind:?}").unwrap();
            indent.push_str("  ");
        }
        StrStep::Exit => {
            assert!(depth > 0);
            depth -= 1;
            indent.pop();
            indent.pop();
        }
        StrStep::Error { msg, pos } => {
            assert!(depth > 0);
            errors.push(format!("error {pos}: {msg}\n"))
        }
    });
    assert_eq!(
        len,
        text.len(),
        "didn't parse all text.\nParsed:\n{}\n\nAll:\n{}\n",
        &text[..len],
        text
    );

    for (token, msg) in lexed.errors() {
        let pos = lexed.text_start(token);
        errors.push(format!("error {pos}: {msg}\n"));
    }

    let has_errors = !errors.is_empty();
    for e in errors {
        buf.push_str(&e);
    }
    (buf, has_errors)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct TestCase {
    sal: PathBuf,
    sast: PathBuf,
    text: String,
}

impl TestCase {
    fn list(path: &'static str) -> Vec<TestCase> {
        let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let test_data_dir = crate_root_dir.join("test_data");
        let dir = test_data_dir.join(path);

        let mut res = Vec::new();
        let read_dir = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("can't `read_dir` {}: {err}", dir.display()));
        for file in read_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.extension().unwrap_or_default() == "sal" {
                let sal = path;
                let sast = sal.with_extension("sast");
                let text = fs::read_to_string(&sal).unwrap();
                res.push(TestCase { sal, sast, text });
            }
        }
        res.sort();
        res
    }
}
