pub mod structures;
pub mod utils;

use scanner::data::{Token, TokenType};
use std::collections::VecDeque;
use structures::{DeclKind, ExpKind, ExpType, Node, ParseError, StmtKind, TreeNode};

fn _match(token: TokenType, tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> bool {
    let current_token = get_current_token(tokens).cloned();
    match current_token {
        Some(c_token) => {
            if c_token.token_type != token {
                errors.push(ParseError {
                    message: format!("Se esperaba un token del tipo {:?}", token),
                    expected_token_type: Some(vec![token]),
                    current_token: Some(c_token),
                });
                return false;
            }
        }
        None => {
            errors.push(ParseError {
                message: format!(
                    "Se esperaba un token del tipo {:?} pero ya no hay ningún token disponible!",
                    token
                ),
                expected_token_type: Some(vec![token]),
                current_token: None,
            });
            return false;
        }
    }

    get_next_token(tokens);
    true
}

fn get_current_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn get_next_token(tokens: &mut VecDeque<Token>) -> Option<Token> {
    tokens.pop_front()
}

pub fn parse(tokens: Vec<Token>) -> (Option<TreeNode>, Vec<ParseError>) {
    let mut deque = VecDeque::from(tokens);
    let mut errors = Vec::new();
    let root = programa(&mut deque, &mut errors);
    (root, errors)
}

fn programa(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    _match(TokenType::MAIN, tokens, errors);
    _match(TokenType::LBRA, tokens, errors);

    let root = lista_declaracion(tokens, errors);

    if root.is_none() {
        errors.push(ParseError {
            message: "Se esperaba una declaración".to_string(),
            expected_token_type: None,
            current_token: get_current_token(tokens).cloned(),
        });
    }

    _match(TokenType::RBRA, tokens, errors);

    root
}

fn lista_declaracion(
    tokens: &mut VecDeque<Token>,
    errors: &mut Vec<ParseError>,
) -> Option<TreeNode> {
    let mut t = declaracion(tokens, errors);

    if let Some(ta) = &mut t {
        let mut last_sib = ta.get_last_sibling_mut();
        while let Some(q_node) = declaracion(tokens, errors) {
            last_sib.sibling = Some(Box::new(q_node));
            last_sib = last_sib.get_last_sibling_mut();
        }
    }
    t
}

fn declaracion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    match get_current_token(tokens) {
        Some(token) => match token.token_type {
            TokenType::INTEGER | TokenType::DOUBLE => declaracion_variable(tokens, errors),
            _ => lista_sentencias(tokens, errors),
        },
        None => None,
    }
}

fn declaracion_variable(
    tokens: &mut VecDeque<Token>,
    errors: &mut Vec<ParseError>,
) -> Option<TreeNode> {
    _match(
        get_current_token(tokens).unwrap().token_type.clone(), // es seguro
        tokens,
        errors,
    );
    let node = identificador(tokens, errors);
    _match(TokenType::SCOL, tokens, errors);
    node
}

fn identificador(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    match get_current_token(&tokens.clone()) {
        Some(token) => {
            if !_match(TokenType::ID, tokens, errors) {
                return None;
            }
            let mut node = TreeNode::new(Node::Decl(DeclKind::Var {
                typ: token.token_type.clone(),
                name: token.lexemme.clone(),
            }));
            let mut current_node = &mut node;

            while let Some(Token {
                token_type: TokenType::COMMA,
                ..
            }) = get_current_token(tokens)
            {
                _match(TokenType::COMMA, tokens, errors);
                let cloned_tokens = tokens.clone();
                let token_op = get_current_token(&cloned_tokens);
                if !_match(TokenType::ID, tokens, errors) || token_op.is_none() {
                    break;
                }
                let sibling_node = TreeNode::new(Node::Decl(DeclKind::Var {
                    typ: token.token_type.clone(),
                    name: token_op.unwrap().lexemme.clone(), // seguro
                }));
                current_node.sibling = Some(Box::new(sibling_node));
                current_node = current_node.sibling.as_mut().unwrap();
            }

            Some(node)
        }
        None => None, // no estoy seguro
    }
}

fn lista_sentencias(
    tokens: &mut VecDeque<Token>,
    errors: &mut Vec<ParseError>,
) -> Option<TreeNode> {
    let mut node = sentencia(tokens, errors);
    let mut current_node = node.as_mut();

    while let Some(s) = sentencia(tokens, errors) {
        if let Some(cn) = current_node {
            cn.sibling = Some(Box::new(s));
            current_node = cn.sibling.as_deref_mut();
        } else {
            break;
            /*  node = Some(s);
            current_node = node.as_mut(); */
        }
    }

    node
}

fn sentencia(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let token = get_current_token(tokens).cloned();
    if token.is_none() {
        // si es none, salir
        return None;
    }

    let curr_token = token.as_ref().unwrap();
    match curr_token.token_type {
        // es seguro
        TokenType::IF => seleccion(tokens, errors),
        TokenType::WHILE => iteracion(tokens, errors),
        TokenType::DO => repeticion(tokens, errors),
        TokenType::STDIN => sent_in(tokens, errors),
        TokenType::STDOUT => sent_out(tokens, errors),
        TokenType::ID => asignacion(tokens, errors),
        TokenType::INTEGER | TokenType::DOUBLE => declaracion_variable(tokens, errors),
        _ => {
            if curr_token.token_type != TokenType::RBRA {
                errors.push(ParseError {
                    current_token: Some(token.unwrap().clone()),
                    message: "Mala sentencia, se esperaba uno de los siguientes tokens: IF, WHILE, DO, STDIN, STDOUT, ID".to_string(),
                    expected_token_type: Some(vec![TokenType::IF, TokenType::WHILE, TokenType::DO, TokenType::STDIN, TokenType::STDOUT, TokenType::ID]),
                });
            }
            None
        }
    }
}

fn asignacion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let token = get_current_token(tokens).unwrap().clone();
    let name = token.lexemme.clone();
    if !_match(TokenType::ID, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba un identificador".to_string(),
            expected_token_type: Some(vec![TokenType::ID]),
            current_token: Some(token.clone()),
        });
        return None;
    }
    if !_match(TokenType::ASSIGN, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '='".to_string(),
            expected_token_type: Some(vec![TokenType::ASSIGN]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let value = sent_expresion(tokens, errors)?;
    Some(TreeNode::new(Node::Stmt(StmtKind::Assign {
        name,
        value: Box::new(value.node),
    })))
}

fn sent_expresion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let node = expresion(tokens, errors);
    if !_match(TokenType::SCOL, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba ';'".to_string(),
            expected_token_type: Some(vec![TokenType::SCOL]),
            current_token: get_current_token(tokens).cloned(),
        });
    }
    node
}

fn seleccion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    if !_match(TokenType::IF, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba 'if'".to_string(),
            expected_token_type: Some(vec![TokenType::IF]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let condition = expresion(tokens, errors)?;
    if !_match(TokenType::LBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '{'".to_string(),
            expected_token_type: Some(vec![TokenType::LBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let then_branch = lista_sentencias(tokens, errors)?;
    if !_match(TokenType::RBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '}'".to_string(),
            expected_token_type: Some(vec![TokenType::RBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let else_branch = if let Some(Token {
        token_type: TokenType::ELSE,
        ..
    }) = get_current_token(tokens)
    {
        _match(TokenType::ELSE, tokens, errors);
        _match(TokenType::LBRA, tokens, errors);
        let r = lista_sentencias(tokens, errors);
        _match(TokenType::RBRA, tokens, errors);
        r
    } else {
        None
    };
    Some(TreeNode::new(Node::Stmt(StmtKind::If {
        condition: Box::new(condition.node),
        then_branch: Box::new(then_branch),
        else_branch: else_branch.map(|n| Box::new(n)),
    })))
}

fn iteracion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    if !_match(TokenType::WHILE, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba 'while'".to_string(),
            expected_token_type: Some(vec![TokenType::WHILE]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let condition = expresion(tokens, errors)?;
    if !_match(TokenType::LBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '{'".to_string(),
            expected_token_type: Some(vec![TokenType::LBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let body = lista_sentencias(tokens, errors)?;
    if !_match(TokenType::RBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '}'".to_string(),
            expected_token_type: Some(vec![TokenType::RBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    Some(TreeNode::new(Node::Stmt(StmtKind::While {
        condition: Box::new(condition.node),
        body: Box::new(body),
    })))
}

fn repeticion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    if !_match(TokenType::DO, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba 'do'".to_string(),
            expected_token_type: Some(vec![TokenType::DO]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    if !_match(TokenType::LBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '{'".to_string(),
            expected_token_type: Some(vec![TokenType::LBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let body = lista_sentencias(tokens, errors)?;
    if !_match(TokenType::RBRA, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba '}'".to_string(),
            expected_token_type: Some(vec![TokenType::RBRA]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    if !_match(TokenType::WHILE, tokens, errors) {
        errors.push(ParseError {
            message: "Se esperaba 'while'".to_string(),
            expected_token_type: Some(vec![TokenType::WHILE]),
            current_token: get_current_token(tokens).cloned(),
        });
        return None;
    }
    let condition = expresion(tokens, errors)?;
    _match(TokenType::SCOL, tokens, errors);
    Some(TreeNode::new(Node::Stmt(StmtKind::Do {
        body: Box::new(body),
        condition: Box::new(condition.node),
    })))
}

fn sent_in(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    _match(TokenType::STDIN, tokens, errors);
    let token_list = tokens.clone();
    let curr_token = get_current_token(&token_list);
    _match(TokenType::ID, tokens, errors);
    if curr_token.is_none() {
        return None;
    }
    let name = curr_token.unwrap().lexemme.clone();
    _match(TokenType::SCOL, tokens, errors);
    Some(TreeNode::new(Node::Stmt(StmtKind::In { name })))
}

fn sent_out(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    _match(TokenType::STDOUT, tokens, errors);
    let expression = expresion(tokens, errors);
    if expression.is_none() {
        errors.push(ParseError {
            current_token: get_current_token(tokens).cloned(),
            expected_token_type: None,
            message: "Se esperaba una expresion".to_string(),
        });
        return None;
    }
    _match(TokenType::SCOL, tokens, errors);
    Some(TreeNode::new(Node::Stmt(StmtKind::Out {
        expression: Box::new(expression.unwrap().node),
    })))
}

fn expresion(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let mut node = expresion_simple(tokens, errors)?;

    if let Some(token) = get_current_token(tokens) {
        match &token.token_type {
            &TokenType::LT
            | &TokenType::LE
            | &TokenType::GT
            | &TokenType::GE
            | &TokenType::EQ
            | &TokenType::NE => {
                let op = token.token_type.clone();
                _match(op.clone(), tokens, errors);
                let right = expresion_simple(tokens, errors)?;
                node = TreeNode::new(Node::Exp {
                    typ: ExpType::Void,
                    kind: ExpKind::Op {
                        op,
                        left: Box::new(node.node),
                        right: Box::new(right.node),
                    },
                });
            }
            _ => {}
        }
    }

    Some(node)
}

fn expresion_simple(
    tokens: &mut VecDeque<Token>,
    errors: &mut Vec<ParseError>,
) -> Option<TreeNode> {
    let mut node = termino(tokens, errors)?;
    while matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::SUM | TokenType::MIN | TokenType::INT | TokenType::FLOAT
    ) {
        let curr = get_current_token(tokens).unwrap();
        let op = curr.token_type.clone();
        match op {
            TokenType::SUM | TokenType::MIN => {
                _match(op.clone(), tokens, errors);
                let right = termino(tokens, errors)?;
                node = TreeNode::new(Node::Exp {
                    typ: ExpType::Void,
                    kind: ExpKind::Op {
                        op,
                        left: Box::new(node.node),
                        right: Box::new(right.node),
                    },
                });
            }
            TokenType::INT | TokenType::FLOAT => {
                if curr.lexemme.contains('+') {
                    let right = termino(tokens, errors)?;
                    node = TreeNode::new(Node::Exp {
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op: TokenType::SUM,
                            left: Box::new(node.node),
                            right: Box::new(right.node),
                        },
                    });
                } else if curr.lexemme.contains('-') {
                    let right = termino(tokens, errors)?;
                    node = TreeNode::new(Node::Exp {
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op: TokenType::MIN,
                            left: Box::new(node.node),
                            right: Box::new(right.node),
                        },
                    });
                } else {
                    errors.push(ParseError {
                        current_token: Some(curr.clone()),
                        expected_token_type: Some(vec![TokenType::SUM, TokenType::MIN]),
                        message: "Se esperaba un símbolo de suma o resta".to_string(),
                    });
                    return None;
                }
            }
            _ => {}
        }
    }
    Some(node)
}

fn termino(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let mut node = factor(tokens, errors)?;
    while matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::TIMES | TokenType::DIV | TokenType::MODULUS
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens, errors);
        let right = factor(tokens, errors)?;
        node = TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(node.node),
                right: Box::new(right.node),
            },
        });
    }
    Some(node)
}

fn factor(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let mut node = componente(tokens, errors)?;
    while matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::POWER
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens, errors);
        let right = componente(tokens, errors)?;
        node = TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(node.node),
                right: Box::new(right.node),
            },
        });
    }
    Some(node)
}

fn componente(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    match get_current_token(tokens) {
        Some(token) => match token.token_type {
            TokenType::LPAR => {
                _match(TokenType::LPAR, tokens, errors);
                let node = expresion(tokens, errors)?;
                _match(TokenType::RPAR, tokens, errors);
                Some(node)
            }
            TokenType::INT => {
                let value: i32 = token.lexemme.parse().unwrap();
                _match(TokenType::INT, tokens, errors);
                Some(TreeNode::new(Node::Exp {
                    kind: ExpKind::Const { value },
                    typ: ExpType::Void,
                }))
            }
            TokenType::FLOAT => {
                let value: f32 = token.lexemme.parse().unwrap();
                _match(TokenType::FLOAT, tokens, errors);
                Some(TreeNode::new(Node::Exp {
                    kind: ExpKind::ConstF { value },
                    typ: ExpType::Void,
                }))
            }
            TokenType::ID => incremento(tokens, errors),
            _ => {
                let expected_token_type = vec![TokenType::LPAR, TokenType::INT, TokenType::ID];
                errors.push(ParseError {
                    message: format!(
                        "Expresión no válida. Se esperaba uno de los siguientes tokens: {:?}",
                        expected_token_type
                    ),
                    current_token: Some(token.clone()),
                    expected_token_type: Some(expected_token_type),
                });
                None
            }
        },
        None => None,
    }
}

fn incremento(tokens: &mut VecDeque<Token>, errors: &mut Vec<ParseError>) -> Option<TreeNode> {
    let name = get_current_token(tokens).unwrap().lexemme.clone();
    _match(TokenType::ID, tokens, errors);
    if matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::INC | TokenType::DEC
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens, errors);
        Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(Node::Exp {
                    typ: ExpType::Void,
                    kind: ExpKind::Id { name },
                }),
                right: Box::new(Node::Exp {
                    kind: ExpKind::Const { value: 1 },
                    typ: ExpType::Void,
                }),
            },
        }))
    } else {
        Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Id { name },
        }))
    }
}
