#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConstexprValue(Option<i32>);

struct EvalContext {}
