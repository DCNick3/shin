use shin_core::format::scenario::instructions::MemoryAddress;

pub enum Preserve {
    Single(MemoryAddress),
    Range(MemoryAddress, MemoryAddress),
}

pub struct Preserves(pub Vec<Preserve>);

pub struct Function {
    pub arg_aliases: Vec<String>, // TODO
    pub preserves: Preserves,
}

// struct S
