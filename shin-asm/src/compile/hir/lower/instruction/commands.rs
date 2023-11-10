#![allow(non_snake_case)]

use shin_core::{
    format::{
        scenario::instruction_elements::{MessageId, NumberSpec, U8Bool},
        text::U16FixupString,
    },
    time::Ticks,
    vm::command::{
        compiletime::{EXIT, MSGINIT, MSGSET, WAIT},
        types::MessageboxStyle,
        CompiletimeCommand,
    },
};

use crate::compile::hir::lower::instruction::router::{Router, RouterBuilder};

fn EXIT() -> CompiletimeCommand {
    CompiletimeCommand::EXIT(EXIT {
        arg1: 0,
        arg2: NumberSpec::constant(1),
    })
}

fn WAIT((wait_amount,): (NumberSpec<Ticks>,)) -> CompiletimeCommand {
    CompiletimeCommand::WAIT(WAIT {
        allow_interrupt: U8Bool(false),
        wait_amount,
    })
}

fn MSGINIT((messagebox_style,): (NumberSpec<MessageboxStyle>,)) -> CompiletimeCommand {
    CompiletimeCommand::MSGINIT(MSGINIT { messagebox_style })
}

fn MSGSET((text,): (U16FixupString,)) -> CompiletimeCommand {
    // TODO: actually MSGSET should be a really special one
    // we want to automatically allocate message ids
    // and we want the auto_wait flag specification to be optional
    // (`@no_wait`?)
    CompiletimeCommand::MSGSET(MSGSET {
        // TODO: no!!!!
        msg_id: MessageId(0),
        auto_wait: U8Bool(true),
        text,
    })
}

pub fn commands(builder: RouterBuilder<impl Router>) -> RouterBuilder<impl Router> {
    builder
        .add("EXIT", EXIT)
        .add("WAIT", WAIT)
        .add("MSGINIT", MSGINIT)
        .add("MSGSET", MSGSET)
}
