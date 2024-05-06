#[derive(Debug)]
pub enum Expr {
    Boolean(bool),
    Integer(u32),
    String(String),
    Identifier(String),
    AssignmentExpr(String, Box<Expr>),
    FunctionApplication(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
}
