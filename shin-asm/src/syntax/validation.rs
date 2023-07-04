use crate::syntax::{SyntaxError, SyntaxNode};

pub(crate) fn validate(root: &SyntaxNode) -> Vec<SyntaxError> {
    let mut errors = Vec::new();
    for node in root.descendants() {
        // TODO: actually perform some validation
    }
    errors
}
