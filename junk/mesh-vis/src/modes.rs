use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Mode {
    Mesh,
    Matrix,
    // TODO: uniform buffer mode
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Mesh => write!(f, "Mesh"),
            Mode::Matrix => write!(f, "Matrix"),
        }
    }
}
