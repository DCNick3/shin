mod syntax_kind;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct ShinAsmParser;

#[cfg(test)]
mod test {
    use super::{Rule, ShinAsmParser};
    use pest::Parser;

    #[test]
    fn test() {
        let pairs = ShinAsmParser::parse(Rule::identifier, "test").unwrap();
        let pairs = ShinAsmParser::parse(Rule::identifier, "a123").unwrap();
        let pairs = ShinAsmParser::parse(Rule::generic_instruction, "aboba 123.0, 12").unwrap();
        // let pairs = ShinAsmParser::parse(Rule::, "123").unwrap();
        println!("{:#?}", pairs);
    }
}
