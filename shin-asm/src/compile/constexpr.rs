#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstexprValue(Option<i32>);

struct EvalContext {}
