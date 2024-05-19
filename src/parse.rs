pub mod structures;
pub mod utils;

use scanner::data::{Token, TokenType};
use std::collections::VecDeque;

use structures::{DeclKind, ExpKind, ExpType, Node, ScanError, StmtKind, TreeNode};

fn _match(token: TokenType, tokens: &mut VecDeque<Token>) -> Result<(), ScanError> {
    let current_token = get_current_token(tokens).unwrap().clone();
    if current_token.token_type != token {
        return Err(ScanError {
            message: format!("Se estaba esperando un token del tipo {:?}", token).to_string(),
            expected_token_type: Some(vec![token]),
            current_token,
        });
    }
    get_next_token(tokens);
    Ok(())
}

fn get_current_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn get_next_token(tokens: &mut VecDeque<Token>) -> Option<Token> {
    tokens.pop_front()
}

pub fn parse(tokens: Vec<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut deque = VecDeque::from(tokens);
    let root = programa(&mut deque);
    Ok(root?)
}

fn programa(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::MAIN, tokens)?;
    _match(TokenType::LBRA, tokens)?;
    let node = lista_declaracion(tokens)?;
    _match(TokenType::RBRA, tokens)?;
    Ok(node)
}

fn lista_declaracion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut t = declaracion(tokens)?;
    let mut p = t.as_mut();
    while let Some(_) = p {
        let q = declaracion(tokens)?;
        if let Some(q_node) = q {
            if let Some(p_node) = p {
                p_node.sibling = Some(Box::new(q_node));
                p = p_node.sibling.as_deref_mut();
            }
        } else {
            break;
        }
    }
    Ok(t)
}

fn declaracion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let t = match get_current_token(tokens) {
        Some(token) => match token.token_type {
            TokenType::INT | TokenType::DOUBLE => declaracion_variable(tokens),
            _ => lista_sentencias(tokens),
        },
        None => todo!(),
    };
    t
}

fn declaracion_variable(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(
        get_current_token(tokens).unwrap().token_type.clone(),
        tokens,
    )?; // consumir token integer o double que ya se comprobo en la regla anterior
    let name = get_current_token(tokens).unwrap().lexemme.clone();
    _match(TokenType::ID, tokens)?;
    identificador(name.clone(), tokens).map(|decls| {
        let mut t = TreeNode::new(Node::Decl(DeclKind::Var {
            typ: get_current_token(tokens).unwrap().token_type.clone(),
            name: name,
        }));
        t.sibling = Some(Box::new(decls.unwrap()));
        _match(TokenType::SCOL, tokens);
        Some(t)
    })
}

fn identificador(
    first_name: String,
    tokens: &mut VecDeque<Token>,
) -> Result<Option<TreeNode>, ScanError> {
    let token = get_current_token(tokens).unwrap();
    let mut t = TreeNode::new(Node::Decl(DeclKind::Var {
        typ: token.token_type.clone(),
        name: first_name,
    }));
    while get_current_token(tokens).unwrap().token_type == TokenType::COMMA {
        _match(TokenType::COMMA, tokens)?;
        let name = get_current_token(tokens).unwrap().lexemme.clone();
        _match(TokenType::ID, tokens)?;
        let sibling = TreeNode::new(Node::Decl(DeclKind::Var {
            typ: get_current_token(tokens).unwrap().token_type.clone(),
            name,
        }));
        t.sibling = Some(Box::new(sibling));
        t = *t.sibling.unwrap();
    }
    Ok(Some(t))
}

fn lista_sentencias(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut t = sentencia(tokens)?;
    let mut p = t.as_mut();
    while let Some(_) = p {
        let q = sentencia(tokens)?;
        if let Some(q_node) = q {
            if let Some(p_node) = p {
                p_node.sibling = Some(Box::new(q_node));
                p = p_node.sibling.as_deref_mut();
            }
        } else {
            break;
        }
    }
    Ok(t)
}

fn sentencia(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let token = get_next_token(tokens).unwrap();
    match token.token_type {
        TokenType::IF => seleccion(tokens),
        TokenType::WHILE => iteracion(tokens),
        TokenType::DO => repeticion(tokens),
        TokenType::STDIN => sent_in(tokens),
        TokenType::STDOUT => sent_out(tokens),
        TokenType::ID => asignacion(tokens),
        _ => {
            let expected = vec![
                TokenType::IF,
                TokenType::WHILE,
                TokenType::DO,
                TokenType::STDIN,
                TokenType::STDOUT,
                TokenType::ID,
            ];
            get_next_token(tokens);
            Err(ScanError {
                current_token: token.clone(),
                message: format!(
                    "Mala sentencia, se esperaba uno de los siguientes tokens: {:?}",
                    expected
                ),
                expected_token_type: Some(expected),
            })
        }
    }
}

fn asignacion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let token = get_current_token(tokens).unwrap();
    let name = token.lexemme.clone();
    _match(TokenType::ID, tokens)?;
    _match(TokenType::ASSIGN, tokens)?;
    let value = sent_expresion(tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::Assign {
        name,
        value: Box::new(value.unwrap().node),
    }))))
}

fn sent_expresion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let t = expresion(tokens);
    _match(TokenType::SCOL, tokens)?;
    t
}

fn seleccion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::IF, tokens)?;
    let condition = expresion(tokens)?;
    _match(TokenType::LBRA, tokens)?;
    let then_branch = sentencia(tokens)?;
    let else_branch = if get_current_token(tokens).unwrap().token_type == TokenType::ELSE {
        _match(TokenType::ELSE, tokens)?;
        _match(TokenType::LBRA, tokens)?;
        let r = Some(Box::new(sentencia(tokens)?));
        _match(TokenType::RBRA, tokens)?;
        r
    } else {
        None
    };
    // _match(TokenType::END, tokens);
    _match(TokenType::RBRA, tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::If {
        condition: Box::new(condition.unwrap().node),
        then_branch: Box::new(then_branch.unwrap().node),
        else_branch: else_branch.map(|n| Box::new(n.unwrap().node)),
    }))))
}

fn iteracion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::WHILE, tokens)?;
    let condition = expresion(tokens)?;
    _match(TokenType::LBRA, tokens)?;
    let body = sentencia(tokens)?;
    _match(TokenType::RBRA, tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::While {
        condition: Box::new(condition.unwrap().node),
        body: Box::new(body.unwrap().node),
    }))))
}

fn repeticion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::DO, tokens)?;
    _match(TokenType::LBRA, tokens)?;
    let body = sentencia(tokens)?;
    _match(TokenType::RBRA, tokens)?;
    _match(TokenType::WHILE, tokens)?;
    let condition = expresion(tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::Do {
        body: Box::new(body.unwrap().node),
        condition: Box::new(condition.unwrap().node),
    }))))
}

fn sent_in(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::STDIN, tokens)?;
    let name = get_current_token(tokens).unwrap().lexemme.clone();
    _match(TokenType::ID, tokens)?;
    _match(TokenType::SCOL, tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::In { name }))))
}

fn sent_out(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    _match(TokenType::STDIN, tokens)?;
    let expression = expresion(tokens)?;
    _match(TokenType::SCOL, tokens)?;
    Ok(Some(TreeNode::new(Node::Stmt(StmtKind::Out {
        expression: Box::new(expression.unwrap().node),
    }))))
}

fn expresion(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut t = expresion_simple(tokens)?;
    let token_type = get_current_token(tokens).unwrap().token_type.clone();
    if matches!(
        token_type,
        TokenType::LT
            | TokenType::LE
            | TokenType::GT
            | TokenType::GE
            | TokenType::EQ
            | TokenType::NE
    ) {
        let op = token_type;
        _match(op.clone(), tokens)?;
        let right = expresion_simple(tokens)?;
        t = Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(t.unwrap().node),
                right: Box::new(right.unwrap().node),
            },
        }));
    }
    Ok(t)
}

fn expresion_simple(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut t = termino(tokens)?;
    while matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::SUM | TokenType::MIN
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens)?;
        let right = termino(tokens)?;
        t = Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(t.unwrap().node),
                right: Box::new(right.unwrap().node),
            },
        }));
    }
    Ok(t)
}

fn termino(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let mut t = factor(tokens)?;

    while matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::TIMES | TokenType::POWER | TokenType::MODULUS
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens)?;
        let right = factor(tokens)?;
        t = Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Op {
                op,
                left: Box::new(t.unwrap().node),
                right: Box::new(right.unwrap().node),
            },
        }));
    }
    Ok(t)
}

fn factor(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let token = get_current_token(tokens).unwrap();
    match token.token_type {
        TokenType::LPAR => {
            _match(TokenType::LPAR, tokens)?;
            let t = expresion(tokens);
            _match(TokenType::RPAR, tokens)?;
            t
        }
        TokenType::INT => {
            let value: i32 = token.lexemme.parse().unwrap();
            _match(TokenType::INT, tokens)?;
            Ok(Some(TreeNode::new(Node::Exp {
                kind: ExpKind::Const { value },
                typ: ExpType::Void,
            })))
        }
        TokenType::ID => incremento(tokens),
        _ => {
            let expected_token_type = vec![TokenType::LPAR, TokenType::INT, TokenType::ID];
            Err(ScanError {
                message: format!(
                    "Expresion no válida. Se esperaba alguno de los siguientes tokens: {:?}",
                    expected_token_type
                ),
                current_token: token.clone(),
                expected_token_type: Some(expected_token_type),
            })
        }
    }
}

fn incremento(tokens: &mut VecDeque<Token>) -> Result<Option<TreeNode>, ScanError> {
    let name = get_current_token(tokens).unwrap().lexemme.clone();
    _match(TokenType::ID, tokens)?;
    if matches!(
        get_current_token(tokens).unwrap().token_type,
        TokenType::INC | TokenType::DEC
    ) {
        let op = get_current_token(tokens).unwrap().token_type.clone();
        _match(op.clone(), tokens)?;
        Ok(Some(TreeNode::new(Node::Exp {
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
        })))
    } else {
        Ok(Some(TreeNode::new(Node::Exp {
            typ: ExpType::Void,
            kind: ExpKind::Id { name },
        })))
    }
}
