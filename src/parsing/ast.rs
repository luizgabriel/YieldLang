#[derive(Debug)]
pub enum Expr {
    Boolean(bool),
    Integer(u32),
    String(String),
    Identifier(String),
    FunctionApplication(Box<Expr>, Box<Expr>),
}
