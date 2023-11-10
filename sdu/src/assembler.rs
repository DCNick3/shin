use std::io::{Read, Seek, SeekFrom};

use anyhow::{Context, Result};
use binrw::BinRead;
use camino::Utf8PathBuf;
use shin_asm::compile::{
    diagnostics::{AriadneDbCache, HirDiagnosticAccumulator, SourceDiagnosticAccumulator},
    hir, File, Program,
};
use shin_core::format::scenario::ScenarioHeader;

#[derive(clap::Subcommand, Debug)]
pub enum AssemblerCommand {
    /// Lex the input file and dump the tokens
    LexDump {
        /// Input file
        input: Utf8PathBuf,
    },
    /// Build an SNR file from source files
    Build {
        /// List of input `.sal` files
        inputs: Vec<Utf8PathBuf>,
        /// A donor `.snr` file whose headers & info tables will be used for the output file
        // TODO: ideally we should support our own textual & easy to tinker format
        //       for defining info tables but making it __good__ will take some time,
        //       so we opt into this "easy" solution for now
        #[clap(long)]
        headers_from: Utf8PathBuf,
        /// Output `.snr` file
        #[clap(short, long, default_value = "main.snr")]
        output: Utf8PathBuf,
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
        AssemblerCommand::Build {
            inputs,
            headers_from,
            output,
        } => {
            let mut headers_from = std::fs::File::open(&headers_from)
                .with_context(|| format!("Failed to read file {:?}", headers_from))?;
            let snr_header =
                ScenarioHeader::read_le(&mut headers_from).context("Failed to parse")?;
            headers_from.seek(SeekFrom::Start(0))?;
            let mut head_data = vec![0; snr_header.code_offset as usize];
            headers_from.read_exact(&mut head_data)?;
            drop(headers_from);

            let db = shin_asm::compile::db::Database::default();
            let db = &db;

            let donor_headers =
                shin_asm::compile::generate_snr::DonorHeaders::new(db, head_data, snr_header);

            let inputs = inputs
                .into_iter()
                .map(|path| {
                    let contents = std::fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read file {:?}", path))?;
                    let path = path.as_str();
                    Ok(File::new(db, path.to_string(), contents))
                })
                .collect::<Result<Vec<_>>>()
                .context("Failed to read input files")?;

            let program = Program::new(db, inputs);

            let lowered_program = hir::lower::lower_program(db, program);

            let hir_errors =
                hir::lower::lower_program::accumulated::<HirDiagnosticAccumulator>(db, program);
            let source_errors =
                hir::lower::lower_program::accumulated::<SourceDiagnosticAccumulator>(db, program);

            let mut ariadne_errors = Vec::new();
            ariadne_errors.extend(source_errors.into_iter().map(|e| e.into_ariadne(db)));
            ariadne_errors.extend(hir_errors.into_iter().map(|e| e.into_ariadne(db)));

            if !ariadne_errors.is_empty() {
                let mut cache = AriadneDbCache::new(db);

                for error in ariadne_errors {
                    error.eprint(&mut cache).context("Failed to print error")?;
                }
                return Err(anyhow::anyhow!("Compilation failed"));
            }

            let output_bytes =
                shin_asm::compile::generate_snr::generate_snr(db, donor_headers, lowered_program);

            std::fs::write(&output, output_bytes).context("Failed to write output file")?;

            Ok(())
        }
    }
}
