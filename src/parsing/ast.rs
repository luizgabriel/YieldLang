#[derive(Debug)]
pub enum LiteralValue {
    Boolean(bool),
    Integer(i32),
    String(String),
}

#[derive(Debug)]
pub enum Expr {
    Literal(LiteralValue),
    Identifier(String),
    AssignmentExpr(String, Box<Expr>),
    FunctionApplication(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
}
