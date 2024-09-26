use scanner::data::{Cursor, Token, TokenType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ParseError {
    pub current_token: Option<Token>,
    pub expected_token_type: Option<Vec<TokenType>>,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub struct TreeNode {
    //    pub children: Vec<Box<TreeNode>>,
    pub sibling: Option<Box<TreeNode>>,
    pub node: Node,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum Node {
    // Tipo de Nodo
    Stmt {
        kind: StmtKind,
        id: String,
    },
    Exp {
        kind: ExpKind,
        typ: ExpType,
        id: String,
    },
    Decl {
        kind: DeclKind,
        id: String,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DeclKind {
    Var { typ: TokenType, name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StmtKind {
    // Sentencias soportadas
    If {
        condition: Box<Node>,
        then_branch: Option<Box<TreeNode>>,
        else_branch: Option<Box<TreeNode>>,
    },
    While {
        condition: Box<Node>,
        body: Option<Box<TreeNode>>,
    },
    Do {
        body: Option<Box<TreeNode>>,
        condition: Box<Node>,
    },
    Assign {
        name: String,
        value: Box<Node>,
    },
    In {
        name: String,
    },
    Out {
        expression: Box<Node>,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum ExpKind {
    // Uso de operadores, constantes e identificadores
    Op {
        op: TokenType,
        left: Box<Node>,
        right: Option<Box<Node>>,
    },
    Const {
        value: i32,
    },
    ConstF {
        value: f32,
    },
    Id {
        name: String,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum ExpType {
    // Para el tipado
    Void,
    Integer,
    Boolean,
}

pub struct SymbolData {
    pub mem_location: i32,
    pub declaration: Cursor,
    pub usages: Vec<SymbolReference>,
}

pub struct SymbolReference {
    pub cursor: Cursor,
}
