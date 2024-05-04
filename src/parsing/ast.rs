#[derive(Debug)]
pub enum Expr {
    Integer(u32),
    String(String),
    Identifier(String),
    FunctionApplication(Box<Expr>, Box<Expr>),
}
