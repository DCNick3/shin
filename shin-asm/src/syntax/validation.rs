use crate::syntax::{SyntaxError, SyntaxNode};

pub(crate) fn validate(root: &SyntaxNode) -> Vec<SyntaxError> {
    #[allow(unused_mut)]
    let mut errors = Vec::new();
    for _node in root.descendants() {
        // TODO: actually perform some validation
    }
    errors
}
