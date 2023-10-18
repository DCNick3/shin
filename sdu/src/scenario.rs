use anyhow::{Context, Result};
use bytes::Bytes;
use itertools::Itertools;
use shin_core::format::scenario::instruction_elements::CodeAddress;
use shin_core::vm::command::{CommandResult, RuntimeCommand};
use std::fs::File;
use std::path::PathBuf;

#[derive(clap::Subcommand, Debug)]
pub enum ScenarioCommand {
    /// Run a scenario in VM, printing all the commands executed
    ///
    /// NOTE: this doesn't work too well with SELECT: it always selects the first option
    Trace {
        /// Path to the SNR file
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
        output_filename: Option<PathBuf>,
    },
    /// Run a scenario in VM, parsing all the messages with layout parser (for testing)
    TestLayouter {
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
    },
    CharFrequency {
        scenario_path: PathBuf,
        /// Initial value of the memory cell "0", usually selecting the episode or smth
        #[clap(default_value = "0")]
        init_val: i32,
        #[clap(default_value = "64")]
        top_k: usize,
    },
    /// Dump (known) header information tables from the scenario
    ///
    /// This includes stuff like picture names, sound names, etc.
    DumpInfo {
        scenario_path: PathBuf,
        output_filename: Option<PathBuf>,
    },
    /// Disassemble a scenario into an assembly-like language
    ///
    /// NOTE: the format of the output is not stable yet
    Disassemble {
        scenario_path: PathBuf,
        output_filename: Option<PathBuf>,
    },
}

fn make_output(output_filename: Option<PathBuf>) -> Result<Box<dyn std::io::Write>> {
    match output_filename {
        None => Ok(Box::new(std::io::stdout().lock())),
        Some(filename) => Ok(Box::new(
            File::create(filename).context("Opening output file")?,
        )),
    }
}

fn trace(path: PathBuf, init_val: i32, output_filename: Option<PathBuf>) -> Result<()> {
    let scenario = std::fs::read(path)?;
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

    let mut output = make_output(output_filename)?;

    let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
    let mut result = CommandResult::None;
    loop {
        // NOTE: usually you would want to do something when the VM has returned "Pending"
        // stuff like running game loop to let the command progress...
        let command = vm.run(result)?;
        writeln!(output, "{:08x} {}", vm.position().0, command)
            .context("Writing to the output file")?;
        if let Some(new_result) = command.execute_dummy() {
            result = new_result
        } else {
            break;
        }
    }

    // println!("{:#?}", reader);
    Ok(())
}

fn test_layouter(path: PathBuf, init_val: i32) -> Result<()> {
    let scenario = std::fs::read(path)?;
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

    let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
    let mut result = CommandResult::None;
    loop {
        let command = vm.run(result)?;

        if let RuntimeCommand::MSGSET(msgset) = &command {
            let layouter = shin_core::layout::LayouterParser::new(&msgset.text);
            let commands = layouter.collect::<Vec<_>>();
            println!("{:?}", commands);
        }

        if let Some(new_result) = command.execute_dummy() {
            result = new_result
        } else {
            break;
        }
    }

    Ok(())
}

fn char_frequency(path: PathBuf, init_val: i32, top_k: usize) -> Result<()> {
    let scenario = std::fs::read(path)?;
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

    let mut counter = counter::Counter::<_, u64>::new();

    let mut vm = shin_core::vm::Scripter::new(&scenario, init_val, 42);
    let mut result = CommandResult::None;
    loop {
        // NOTE: usually you would want to do something when the VM has returned "Pending"
        // stuff like running game loop to let the command progress...
        let command = vm.run(result)?;

        if let RuntimeCommand::MSGSET(msgset) = &command {
            let layouter = shin_core::layout::LayouterParser::new(&msgset.text);
            for command in layouter {
                match command {
                    shin_core::layout::ParsedCommand::Char(c) => {
                        counter[&c] += 1;
                    }
                    shin_core::layout::ParsedCommand::Furigana(text) => {
                        counter.update(text.chars());
                    }
                    _ => {}
                }
            }
        }

        if let Some(new_result) = command.execute_dummy() {
            result = new_result
        } else {
            break;
        }
    }

    println!(
        "{:#?}",
        counter
            .k_most_common_ordered(top_k)
            .into_iter()
            .map(|v| v.0)
            .sorted()
            .join("")
    );
    Ok(())
}

fn dump_info(path: PathBuf, output_filename: Option<PathBuf>) -> Result<()> {
    let scenario = std::fs::read(path)?;
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

    let mut output = make_output(output_filename)?;

    let tables = scenario.info_tables();
    // I kinda hate it. Can we have a macro-based solution?

    writeln!(output, "Masks:")?;
    for (i, mask) in tables.mask_info.iter().enumerate() {
        writeln!(output, "  {}: {:?}", i, mask.name)?;
    }
    writeln!(output, "Pictures:")?;
    for (i, picture) in tables.picture_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?}",
            i, picture.name, picture.linked_cg_id
        )?;
    }
    writeln!(output, "Bustups:")?;
    for (i, bustup) in tables.bustup_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?} {:?}",
            i, bustup.name, bustup.emotion, bustup.lipsync_character_id
        )?;
    }
    writeln!(output, "Bgms:")?;
    for (i, bgm) in tables.bgm_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?} {:?}",
            i, bgm.name, bgm.display_name, bgm.linked_bgm_id
        )?;
    }
    writeln!(output, "Ses:")?;
    for (i, se) in tables.se_info.iter().enumerate() {
        writeln!(output, "  {}: {:?}", i, se.name)?;
    }
    writeln!(output, "Movies:")?;
    for (i, movie) in tables.movie_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?} {:?} {:?}",
            i, movie.name, movie.linked_picture_id, movie.flags, movie.linked_picture_id
        )?;
    }
    writeln!(output, "Voice Mappings:")?;
    for (_, mapping) in tables.voice_mapping_info.iter().enumerate() {
        writeln!(
            output,
            "  {:?}: {:?}",
            mapping.name_pattern, mapping.lipsync_character_ids
        )?;
    }
    writeln!(output, "Picture Box Entries:")?;
    for (i, item) in tables.picture_box_info.iter().enumerate() {
        writeln!(output, "  {}: {:?} {:?}", i, item.name, item.picture_ids)?;
    }
    writeln!(output, "Music Box Entries:")?;
    for (i, item) in tables.music_box_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?} {:?}",
            i, item.bgm_id, item.name_index, item.once_flag
        )?;
    }
    writeln!(output, "Character Box Segments:")?;
    for (i, item) in tables.character_box_info.iter().enumerate() {
        writeln!(output, "  {}: {:?}", i, item)?;
    }
    writeln!(output, "Character Sprites:")?;
    for (i, item) in tables.chars_sprite_info.iter().enumerate() {
        writeln!(output, "  {}: {:?} {:?}", i, item.episode, item.segments)?;
    }
    writeln!(output, "Character Grids:")?;
    for (i, item) in tables.chars_grid_info.iter().enumerate() {
        writeln!(output, "  {}: {:?}", i, item.segments)?;
    }
    writeln!(output, "Tips:")?;
    for (i, tip) in tables.tips_info.iter().enumerate() {
        writeln!(
            output,
            "  {}: {:?} {:?} {:?} {:?}",
            i, tip.episode, tip.title_index, tip.title, tip.content
        )?;
    }

    Ok(())
}

fn disassemble(path: PathBuf, output_filename: Option<PathBuf>) -> Result<()> {
    let scenario = std::fs::read(path)?;
    let scenario = Bytes::from(scenario);
    let scenario = shin_core::format::scenario::Scenario::new(scenario)?;

    let mut output = make_output(output_filename)?;

    let entry = scenario.entrypoint_address();
    let mut reader = scenario.instruction_reader(entry);

    let mut end_position = scenario.raw().len();
    // scenario file is aligned to 0x10 bytes, so there are some zeros at the end
    // trim them
    while end_position > 0 && scenario.raw()[end_position - 1] == 0 {
        end_position -= 1;
    }
    let end_position = CodeAddress(end_position as u32);

    while reader.position() < end_position {
        let position = reader.position();

        let instruction = reader
            .read()
            .with_context(|| format!("Reading instruction at {}", position))?;
        writeln!(output, "{:08x?} {:?}", position.0, instruction)?;
    }

    Ok(())
}

pub fn scenario_command(command: ScenarioCommand) -> Result<()> {
    match command {
        ScenarioCommand::Trace {
            scenario_path,
            init_val,
            output_filename,
        } => trace(scenario_path, init_val, output_filename),
        ScenarioCommand::TestLayouter {
            scenario_path,
            init_val,
        } => test_layouter(scenario_path, init_val),
        ScenarioCommand::CharFrequency {
            scenario_path,
            init_val,
            top_k,
        } => char_frequency(scenario_path, init_val, top_k),
        ScenarioCommand::DumpInfo {
            scenario_path,
            output_filename,
        } => dump_info(scenario_path, output_filename),
        ScenarioCommand::Disassemble {
            scenario_path,
            output_filename,
        } => disassemble(scenario_path, output_filename),
    }
}
