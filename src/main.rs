use scanner::data::{Cursor, Token};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanError {
    pub token: Token,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct TreeNode {
    pub child: Vec<Box<TreeNode>>,
    pub sibling: Box<TreeNode>,
    pub node: NodeKind,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum NodeKind {
    // Tipo de Nodo
    StmtK(StmtKind),
    ExpK((ExpKind, ExpType)),
}
#[derive(Serialize, Deserialize, Debug)]

pub enum StmtKind {
    // Distintas sentencias soportadas
    IfK,
    RepeatK,
    AssignK,
    ReadK,
    WriteK,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum ExpKind {
    // Uso de operadores, constantes e identificadores
    OpK,
    ConstK,
    IdK,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum ExpType {
    // Para el tipado
    Void,
    Integer,
    Boolean,
}

pub fn parse(tokens: Vec<Token>) {
    for token in tokens.iter() {}
}

fn main() {
    println!("Hello, world!");
}
