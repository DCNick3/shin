use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum AssemblerCommand {
    /// Lex the input file and dump the tokens
    LexDump {
        /// Input file
        input: String,
    },
}

pub fn assembler_command(command: AssemblerCommand) -> Result<()> {
    match command {
        AssemblerCommand::LexDump { input } => {
            let input = std::fs::read_to_string(input)?;
            let lexed = shin_asm::parser::LexedStr::new(&input);
            for i in 0..lexed.len() {
                let syntax_kind = lexed.kind(i);
                // if syntax_kind.is_trivia() {
                //     continue;
                // }

                let syntax_kind = format!("{:?}", syntax_kind);
                let text = lexed.text(i);

                println!("{:20} {:?}", syntax_kind, text);
            }

            println!("Errors:");
            for (pos, error) in lexed.errors() {
                let text = lexed.text(pos);
                println!("{:?} {:?}", text, error);
            }
            if lexed.errors().next().is_none() {
                println!(" (None)");
            }
            Ok(())
        }
    }
}
