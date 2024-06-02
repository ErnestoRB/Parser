use scanner::data::{Token, TokenType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScanError {
    pub current_token: Option<Token>,
    pub expected_token_type: Option<Vec<TokenType>>,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub struct TreeNode {
    pub children: Vec<Box<TreeNode>>,
    pub sibling: Option<Box<TreeNode>>,
    pub node: Node,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum Node {
    // Tipo de Nodo
    Stmt(StmtKind),
    Exp { kind: ExpKind, typ: ExpType },
    Decl(DeclKind),
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
        then_branch: Box<TreeNode>,
        else_branch: Option<Box<TreeNode>>,
    },
    While {
        condition: Box<Node>,
        body: Box<TreeNode>,
    },
    Do {
        body: Box<TreeNode>,
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
        right: Box<Node>,
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
