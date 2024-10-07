use scanner::data::{Cursor, Token, TokenType};
use serde::{Deserialize, Serialize};

//Valor para nodo exp ya sea Int o Float
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeValue {
    Int(i32),
    Float(f32),
    Boolean(bool),
}

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
        cursor: Option<Cursor>,
    },
    Exp {
        kind: ExpKind,
        typ: ExpType,
        id: String,
        cursor: Option<Cursor>,
        val: Option<NodeValue>,
    },
    Decl {
        kind: DeclKind,
        id: String,
        cursor: Option<Cursor>,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DeclKind {
    Var { typ: ExpType, name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StmtKind {
    // Sentencias soportadas
    If {
        condition: Box<TreeNode>,
        then_branch: Option<Box<TreeNode>>,
        else_branch: Option<Box<TreeNode>>,
    },
    While {
        condition: Box<TreeNode>,
        body: Option<Box<TreeNode>>,
    },
    Do {
        body: Option<Box<TreeNode>>,
        condition: Box<TreeNode>,
    },
    Assign {
        name: String,
        value: Box<TreeNode>,
    },
    In {
        name: String,
    },
    Out {
        expression: Box<TreeNode>,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub enum ExpKind {
    // Uso de operadores, constantes e identificadores
    Op {
        op: TokenType,
        left: Box<TreeNode>,
        right: Option<Box<TreeNode>>,
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
    Float,
    Boolean,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SymbolData {
    pub mem_location: i32,
    pub declaration: Cursor,
    pub typ: ExpType,
    pub value: Option<NodeValue>,
    pub usages: Vec<SymbolReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnalyzeError {
    pub message: String,
    pub cursor: Cursor,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]

pub struct SymbolReference {
    pub cursor: Cursor,
}
