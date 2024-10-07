pub mod structures;
pub mod utils;

use scanner::data::{Cursor, Token, TokenType};
use std::collections::VecDeque;
use structures::{DeclKind, ExpKind, ExpType, Node, ParseError, StmtKind, TreeNode};
use uuid::Uuid;

struct Parser {
    tokens: VecDeque<Token>,
    errors: Vec<ParseError>,
    current_cursor: Option<Cursor>,
}

pub fn parse(tokens: Vec<Token>) -> (Option<TreeNode>, Vec<ParseError>) {
    let parser = Parser::new(VecDeque::from(tokens));
    return parser.parse();
}

impl Parser {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        let current_cursor = tokens.front().map(|t| t.start.clone());
        Parser {
            errors: vec![],
            tokens,
            current_cursor,
        }
    }

    fn _match(&mut self, token: TokenType, handle_error: bool) -> bool {
        let current_token = self.get_current_token().cloned();
        match current_token {
            Some(c_token) => {
                if c_token.token_type != token {
                    if handle_error {
                        self.errors.push(ParseError {
                            message: format!("Se esperaba un token del tipo {:?}", token),
                            expected_token_type: Some(vec![token]),
                            current_token: Some(c_token),
                        });
                    }
                    return false;
                }
            }
            None => {
                if handle_error {
                    self.errors.push(ParseError {
                        message: format!(
                        "Se esperaba un token del tipo {:?} pero ya no hay ningún token disponible!",
                        token
                    ),
                        expected_token_type: Some(vec![token]),
                        current_token: None,
                    });
                }
                return false;
            }
        }

        self.get_next_token();
        true
    }

    fn get_current_token(&mut self) -> Option<&Token> {
        self.tokens.front()
    }

    fn get_next_token(&mut self) -> Option<Token> {
        self.current_cursor = self.tokens.front().map(|t| t.start.clone());
        self.tokens.pop_front()
    }

    fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn avanzar_hasta(&mut self, token: TokenType) -> bool {
        while let Some(tkn) = self.get_next_token() {
            if tkn.token_type == token {
                return true;
            }
        }
        return false;
    }

    pub fn parse(mut self) -> (Option<TreeNode>, Vec<ParseError>) {
        let root = self.programa();
        (root, self.errors)
    }

    fn programa(&mut self) -> Option<TreeNode> {
        if !self._match(TokenType::MAIN, true) {
            return None;
        };
        if !self._match(TokenType::LBRA, true) {
            return None;
        }

        let root = self.lista_declaracion();

        /*  if root.is_none() {
            errors.push(ParseError {
                message: "Se esperaba una declaración".to_string(),
                expected_token_type: None,
                current_token: get_current_token(tokens).cloned(),
            });
        } */

        let current_token = self.get_current_token().cloned();
        let is_rbra = self._match(TokenType::RBRA, true);
        if !is_rbra {
            self.add_error(ParseError {
                message: format!("Falta cerrar la llave del main!",),
                expected_token_type: None,
                current_token,
            });
        };
        let token = self.get_current_token().cloned();
        if let Some(tkn) = token {
            self.add_error(ParseError {
                message: format!("No se puede escribir fuera del cuerpo del main!",),
                expected_token_type: None,
                current_token: Some(tkn.clone()),
            });
        }

        root
    }

    fn lista_declaracion(&mut self) -> Option<TreeNode> {
        let mut t = self.declaracion();

        if let Some(ta) = &mut t {
            let mut last_sib = ta.get_last_sibling_mut();
            while let Some(q_node) = self.declaracion() {
                last_sib.sibling = Some(Box::new(q_node));
                last_sib = last_sib.get_last_sibling_mut();
            }
        }
        t
    }

    fn declaracion(&mut self) -> Option<TreeNode> {
        match self.get_current_token() {
            Some(token) => match token.token_type {
                TokenType::INTEGER | TokenType::DOUBLE => self.declaracion_variable(),
                _ => self.lista_sentencias(),
            },
            None => None,
        }
    }

    fn declaracion_variable(&mut self) -> Option<TreeNode> {
        let typ = self.get_current_token().unwrap().token_type.clone();
        let typ2 = typ.clone();
        self._match(
            typ, // es seguro
            true,
        );
        let exp_type = match typ2 {
            TokenType::INTEGER => ExpType::Integer,
            TokenType::DOUBLE => ExpType::Float,
            _ => ExpType::Void, // no deberia suceder
        };
        let node = self.identificador(exp_type);
        self._match(TokenType::SCOL, true);
        node
    }

    fn identificador(&mut self, typ: ExpType) -> Option<TreeNode> {
        match self.get_current_token().cloned() {
            Some(token) => {
                if !self._match(TokenType::ID, true) {
                    return None;
                }
                let cursor = self.current_cursor.clone();
                let mut node = TreeNode::new(Node::Decl {
                    kind: DeclKind::Var {
                        typ: typ.clone(),
                        name: token.lexemme.clone(),
                    },
                    id: Uuid::new_v4().to_string(),
                    cursor,
                });
                let mut current_node = &mut node;

                while let Some(Token {
                    token_type: TokenType::COMMA,
                    ..
                }) = self.get_current_token().cloned()
                {
                    self._match(TokenType::COMMA, true);
                    let token_op = self.get_current_token().cloned();
                    if !self._match(TokenType::ID, true) || token_op.is_none() {
                        break;
                    }
                    let cursor = self.current_cursor.clone();

                    let sibling_node = TreeNode::new(Node::Decl {
                        cursor,
                        kind: DeclKind::Var {
                            typ: typ.clone(),
                            name: token_op.unwrap().lexemme.clone(), // seguro
                        },
                        id: Uuid::new_v4().to_string(),
                    });
                    current_node.sibling = Some(Box::new(sibling_node));
                    current_node = current_node.sibling.as_mut().unwrap();
                }

                Some(node)
            }
            None => None, // no estoy seguro
        }
    }

    fn lista_sentencias(&mut self) -> Option<TreeNode> {
        let mut node = None;
        let mut current_node: *mut TreeNode = std::ptr::null_mut();

        while !self.tokens.is_empty() {
            let original_len = self.errors.len(); //Hay que checar la longitud de esta lista para ver si mas adelante va a haber errores
            let result = self.sentencia();

            match result {
                Some(new_node) => {
                    if node.is_none() {
                        node = Some(new_node);
                        current_node = node.as_mut().unwrap().get_last_sibling_mut();
                    } else {
                        unsafe {
                            (*current_node).sibling = Some(Box::new(new_node));
                            current_node = (*current_node)
                                .sibling
                                .as_deref_mut()
                                .unwrap()
                                .get_last_sibling_mut();
                        }
                    }
                }
                None => {
                    /* if let Some(tkn) = get_current_token(tokens) {
                        if tkn.token_type == TokenType::RBRA {
                            break;
                        }
                    } else {
                        break;
                    } */
                    if self.errors.len() != original_len {
                        //si aumento el valor de la lista fue porque hubo un error una vez retorna el None
                        //get_next_token(tokens); //entonces ese token no es valido, se va con el siguiente
                    } else {
                        break;
                    }
                }
            }
        }

        node
    }

    fn sentencia(&mut self) -> Option<TreeNode> {
        let token = self.get_current_token().cloned();
        if token.is_none() {
            // si es none, salir
            return None;
        }

        let curr_token = token.as_ref().unwrap();
        match curr_token.token_type {
            // es seguro
            TokenType::IF => self.seleccion(),
            TokenType::WHILE => self.iteracion(),
            TokenType::DO => self.repeticion(),
            TokenType::STDIN => self.sent_in(),
            TokenType::STDOUT => self.sent_out(),
            TokenType::ID => self.asignacion(),
            TokenType::INTEGER | TokenType::DOUBLE => self.declaracion_variable(),
            _ => {
                if curr_token.token_type != TokenType::RBRA {
                    // si no es fin del main
                    self.errors.push(ParseError {
                    current_token: Some(token.unwrap().clone()),
                    message: "Mala sentencia, se esperaba uno de los siguientes tokens: IF, WHILE, DO, STDIN, STDOUT, ID".to_string(),
                    expected_token_type: Some(vec![TokenType::IF, TokenType::WHILE, TokenType::DO, TokenType::STDIN, TokenType::STDOUT, TokenType::ID]),
                });
                    self.get_next_token(); // consumir token invalido
                }
                None
            }
        }
    }

    fn asignacion(&mut self) -> Option<TreeNode> {
        let token = self.get_current_token().unwrap().clone();
        let name = token.lexemme.clone();
        if !self._match(TokenType::ID, false) {
            // no deberia pasar
            self.errors.push(ParseError {
                message: "Se esperaba un identificador".to_string(),
                expected_token_type: Some(vec![TokenType::ID]),
                current_token: Some(token.clone()),
            });
            self.avanzar_hasta(TokenType::SCOL);
            return None;
        }
        let cursor = self.current_cursor.clone();

        match self.get_current_token().cloned() {
            Some(token) => {
                match token.token_type {
                    TokenType::INC | TokenType::DEC => {
                        let operacion = match token.token_type {
                            TokenType::INC => TokenType::SUM,
                            TokenType::DEC => TokenType::MIN,
                            _ => unreachable!("Tipo de token inesperado"),
                        };
                        self._match(token.token_type.clone(), true); // ++ o --
                        let node = Some(TreeNode::new(Node::Stmt {
                            cursor: cursor.clone(),
                            id: Uuid::new_v4().to_string(),
                            kind: StmtKind::Assign {
                                name: name.clone(),
                                value: Box::new(TreeNode::new(Node::Exp {
                                    cursor: cursor.clone(),
                                    typ: ExpType::Void,
                                    kind: ExpKind::Op {
                                        op: operacion,
                                        left: Box::new(TreeNode::new(Node::Exp {
                                            cursor,
                                            typ: ExpType::Void,
                                            kind: ExpKind::Id { name: name.clone() },
                                            id: Uuid::new_v4().to_string(),
                                            val: None,
                                        })),
                                        right: Some(Box::new(TreeNode::new(Node::Exp {
                                            cursor: None,
                                            typ: ExpType::Void,
                                            kind: ExpKind::Const { value: 1 },
                                            id: Uuid::new_v4().to_string(),
                                            val: None,
                                        }))),
                                    },
                                    val: None,
                                    id: Uuid::new_v4().to_string(),
                                })),
                            },
                        }));
                        self._match(TokenType::SCOL, true);
                        return node;
                    }
                    _ => {}
                }
            }
            None => return None,
        }

        if !self._match(TokenType::ASSIGN, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba '='".to_string(),
                expected_token_type: Some(vec![TokenType::ASSIGN]),
                current_token: token,
            });
            self.avanzar_hasta(TokenType::SCOL);
            return None;
        }
        let value = self.sent_expresion()?;
        Some(TreeNode::new(Node::Stmt {
            cursor,
            id: Uuid::new_v4().to_string(),
            kind: StmtKind::Assign {
                name,
                value: Box::new(value),
            },
        }))
    }

    fn sent_expresion(&mut self) -> Option<TreeNode> {
        let node = self.expresion();
        if node.is_none() {
            self.avanzar_hasta(TokenType::SCOL);
            return None;
        }
        if !self._match(TokenType::SCOL, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba ';'".to_string(),
                expected_token_type: Some(vec![TokenType::SCOL]),
                current_token: token,
            });
        }
        node
    }

    fn seleccion(&mut self) -> Option<TreeNode> {
        if !self._match(TokenType::IF, false) {
            // realmente no debería pasar
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba 'if'".to_string(),
                expected_token_type: Some(vec![TokenType::IF]),
                current_token: token,
            });
            return None;
        }
        let cursor = self.current_cursor.clone();

        let condition = self.expresion();
        if condition.is_none() {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba una expresión como condición del if".to_string(),
                expected_token_type: None,
                current_token: token,
            });
            self.avanzar_hasta(TokenType::RBRA); // punto seguro
            if let Some(Token {
                token_type: TokenType::ELSE,
                ..
            }) = self.get_current_token()
            {
                self.avanzar_hasta(TokenType::RBRA); // punto seguro adicional si hay else
            }
            return None;
        }

        if !self._match(TokenType::LBRA, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba '{'".to_string(),
                expected_token_type: None,
                current_token: token,
            });
            self.avanzar_hasta(TokenType::RBRA); // punto seguro
            if let Some(Token {
                token_type: TokenType::ELSE,
                ..
            }) = self.get_current_token()
            {
                self.avanzar_hasta(TokenType::RBRA); // punto seguro adicional si hay else
            }
            return None;
        }

        let then_branch = self.lista_sentencias();
        if !self._match(TokenType::RBRA, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba '}'".to_string(),
                expected_token_type: Some(vec![TokenType::RBRA]),
                current_token: token,
            });
            self.avanzar_hasta(TokenType::RBRA); // punto seguro
            if let Some(Token {
                token_type: TokenType::ELSE,
                ..
            }) = self.get_current_token()
            {
                self.avanzar_hasta(TokenType::RBRA); // punto seguro adicional si hay else
            }
            return None;
        }

        let else_branch = if let Some(Token {
            token_type: TokenType::ELSE,
            ..
        }) = self.get_current_token()
        {
            self._match(TokenType::ELSE, true);
            if !self._match(TokenType::LBRA, false) {
                let token = self.get_current_token().cloned();
                self.errors.push(ParseError {
                    message: "Se esperaba '{' después del 'else'".to_string(),
                    expected_token_type: None,
                    current_token: token,
                });
                self.avanzar_hasta(TokenType::RBRA); // punto seguro adicional si hay else
                return None;
            }
            let r = self.lista_sentencias();
            if !self._match(TokenType::RBRA, false) {
                let token = self.get_current_token().cloned();
                self.errors.push(ParseError {
                    message: "Se esperaba '}' después del bloque 'else'".to_string(),
                    expected_token_type: Some(vec![TokenType::RBRA]),
                    current_token: token,
                });
                return None;
            }
            r
        } else {
            None
        };

        Some(TreeNode::new(Node::Stmt {
            cursor,
            id: Uuid::new_v4().to_string(),
            kind: StmtKind::If {
                condition: Box::new(condition.unwrap()),
                then_branch: then_branch.map(|n| Box::new(n)),
                else_branch: else_branch.map(|n| Box::new(n)),
            },
        }))
    }

    fn iteracion(&mut self) -> Option<TreeNode> {
        if !self._match(TokenType::WHILE, false) {
            let token = self.get_current_token().cloned();
            // realmente no deberia pasar
            self.errors.push(ParseError {
                message: "Se esperaba 'while'".to_string(),
                expected_token_type: Some(vec![TokenType::WHILE]),
                current_token: token,
            });
            return None;
        }
        let cursor = self.current_cursor.clone();

        match self.expresion() {
            Some(condition) => {
                if !self._match(TokenType::LBRA, false) {
                    let token = self.get_current_token().cloned();
                    self.errors.push(ParseError {
                        message: "Se esperaba '{'".to_string(),
                        expected_token_type: Some(vec![TokenType::LBRA]),
                        current_token: token,
                    });
                    self.avanzar_hasta(TokenType::RBRA); // punto seguro

                    return None;
                }
                let body = self.lista_sentencias();
                if !self._match(TokenType::RBRA, false) {
                    let token = self.get_current_token().cloned();
                    self.errors.push(ParseError {
                        message: "Se esperaba '}'".to_string(),
                        expected_token_type: Some(vec![TokenType::RBRA]),
                        current_token: token,
                    });
                    self.avanzar_hasta(TokenType::RBRA); // punto seguro

                    return None;
                }
                Some(TreeNode::new(Node::Stmt {
                    cursor,
                    id: Uuid::new_v4().to_string(),
                    kind: StmtKind::While {
                        condition: Box::new(condition),
                        body: body.map(|n| Box::new(n)),
                    },
                }))
            }
            None => {
                self.avanzar_hasta(TokenType::RBRA);
                None
            }
        }
    }

    fn repeticion(&mut self) -> Option<TreeNode> {
        if !self._match(TokenType::DO, false) {
            let token = self.get_current_token().cloned();
            // realmente no deberia pasar
            self.errors.push(ParseError {
                message: "Se esperaba 'do'".to_string(),
                expected_token_type: Some(vec![TokenType::DO]),
                current_token: token,
            });
            return None;
        }
        let cursor = self.current_cursor.clone();
        if !self._match(TokenType::LBRA, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba '{'".to_string(),
                expected_token_type: Some(vec![TokenType::LBRA]),
                current_token: token,
            });
            self.avanzar_hasta(TokenType::RBRA); // punto seguro
            return None;
        }
        let body = self.lista_sentencias();
        if !self._match(TokenType::RBRA, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba '}'".to_string(),
                expected_token_type: Some(vec![TokenType::RBRA]),
                current_token: token,
            });
            self.avanzar_hasta(TokenType::SCOL); // punto seguro
            return None;
        }
        if !self._match(TokenType::WHILE, false) {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                message: "Se esperaba 'while'".to_string(),
                expected_token_type: Some(vec![TokenType::WHILE]),
                current_token: token,
            });
            self.avanzar_hasta(TokenType::SCOL); // punto seguro
            return None;
        }

        match self.expresion() {
            Some(condition) => {
                self._match(TokenType::SCOL, false);
                Some(TreeNode::new(Node::Stmt {
                    cursor,
                    id: Uuid::new_v4().to_string(),
                    kind: StmtKind::Do {
                        body: body.map(|n| Box::new(n)),
                        condition: Box::new(condition),
                    },
                }))
            }
            None => {
                self.avanzar_hasta(TokenType::SCOL); // punto seguro
                None
            }
        }
    }

    fn sent_in(&mut self) -> Option<TreeNode> {
        self._match(TokenType::STDIN, true); // realmente no deberia pasar
        let cursor = self.current_cursor.clone();
        let curr_token = self.get_current_token().cloned();
        if !self._match(TokenType::ID, true) {
            self.avanzar_hasta(TokenType::SCOL);
            return None;
        }
        if curr_token.is_none() {
            return None;
        }
        let name = curr_token.unwrap().lexemme.clone();
        self._match(TokenType::SCOL, true);
        Some(TreeNode::new(Node::Stmt {
            cursor,
            id: Uuid::new_v4().to_string(),
            kind: StmtKind::In { name },
        }))
    }

    fn sent_out(&mut self) -> Option<TreeNode> {
        self._match(TokenType::STDOUT, true); // realmente no deberia pasar
        let cursor = self.current_cursor.clone();
        let expression = self.expresion();
        if expression.is_none() {
            let token = self.get_current_token().cloned();
            self.errors.push(ParseError {
                current_token: token,
                expected_token_type: None,
                message: "Se esperaba una expresion".to_string(),
            });
            self.avanzar_hasta(TokenType::SCOL);
            return None;
        }
        self._match(TokenType::SCOL, true);
        Some(TreeNode::new(Node::Stmt {
            cursor,
            id: Uuid::new_v4().to_string(),
            kind: StmtKind::Out {
                expression: Box::new(expression.unwrap()),
            },
        }))
    }

    fn expresion(&mut self) -> Option<TreeNode> {
        let mut node = self.expresion_logica_and()?;

        if let Some(token) = self.get_current_token().cloned() {
            match &token.token_type {
                &TokenType::OR => {
                    let op = token.token_type.clone();
                    self._match(op.clone(), true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    let right = self.expresion_logica_and()?;
                    node = TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op,
                            left: Box::new(node),
                            right: Some(Box::new(right)),
                        },
                        val: None,
                    });
                }
                _ => {}
            }
        }

        Some(node)
    }

    fn expresion_logica_and(&mut self) -> Option<TreeNode> {
        let mut node = self.expresion_logica_not()?;

        if let Some(token) = self.get_current_token().cloned() {
            match &token.token_type {
                &TokenType::AND => {
                    let op = token.token_type.clone();
                    self._match(op.clone(), true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    let right = self.expresion_logica_not()?;
                    node = TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op,
                            left: Box::new(node),
                            right: Some(Box::new(right)),
                        },
                        val: None,
                    });
                }
                _ => {}
            }
        }

        Some(node)
    }

    fn expresion_logica_not(&mut self) -> Option<TreeNode> {
        match self.get_current_token() {
            Some(token) => match &token.token_type {
                &TokenType::NEG => {
                    let op = token.token_type.clone();
                    self._match(op.clone(), true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    let left = self.expresion_rel()?;
                    Some(TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            // unario
                            op,
                            left: Box::new(left),
                            right: None,
                        },
                        val: None,
                    }))
                }
                _ => self.expresion_rel(),
            },
            None => None,
        }
    }

    fn expresion_rel(&mut self) -> Option<TreeNode> {
        let mut node = self.expresion_simple()?;

        if let Some(token) = self.get_current_token().cloned() {
            match &token.token_type {
                &TokenType::LT
                | &TokenType::LE
                | &TokenType::GT
                | &TokenType::GE
                | &TokenType::EQ
                | &TokenType::NE => {
                    let op = token.token_type.clone();
                    self._match(op.clone(), true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    let right = self.expresion_simple()?;
                    node = TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op,
                            left: Box::new(node),
                            right: Some(Box::new(right)),
                        },
                        val: None,
                    });
                }
                _ => {}
            }
        }

        Some(node)
    }

    fn expresion_simple(&mut self) -> Option<TreeNode> {
        let mut node = self.termino()?;
        while matches!(
            self.get_current_token().unwrap().token_type,
            TokenType::SUM | TokenType::MIN | TokenType::INT | TokenType::FLOAT
        ) {
            let curr = self.get_current_token().cloned().unwrap();
            let op = curr.token_type.clone();
            match op {
                TokenType::SUM | TokenType::MIN => {
                    self._match(op.clone(), true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    let right = self.termino()?;
                    node = TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Op {
                            op,
                            left: Box::new(node),
                            right: Some(Box::new(right)),
                        },
                        val: None,
                    });
                }
                TokenType::INT | TokenType::FLOAT => {
                    if curr.lexemme.contains('+') || curr.lexemme.contains('-') {
                        let cursor = self.current_cursor.clone();
                        let right = self.termino()?;
                        node = TreeNode::new(Node::Exp {
                            cursor,
                            id: Uuid::new_v4().to_string(),
                            typ: ExpType::Void,
                            kind: ExpKind::Op {
                                op: TokenType::SUM,
                                left: Box::new(node),
                                right: Some(Box::new(right)),
                            },
                            val: None,
                        });
                    } else {
                        self.errors.push(ParseError {
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

    fn termino(&mut self) -> Option<TreeNode> {
        let mut node = self.factor()?;
        while matches!(
            self.get_current_token().unwrap().token_type,
            TokenType::TIMES | TokenType::DIV | TokenType::MODULUS
        ) {
            let op = self.get_current_token().unwrap().token_type.clone();
            self._match(op.clone(), true);
            let cursor = self.current_cursor.clone();
            let right = self.factor()?;

            node = TreeNode::new(Node::Exp {
                cursor,
                id: Uuid::new_v4().to_string(),
                typ: ExpType::Void,
                kind: ExpKind::Op {
                    op,
                    left: Box::new(node),
                    right: Some(Box::new(right)),
                },
                val: None,
            });
        }
        Some(node)
    }

    fn factor(&mut self) -> Option<TreeNode> {
        let mut node = self.componente()?;
        while matches!(
            self.get_current_token().unwrap().token_type,
            TokenType::POWER
        ) {
            let op = self.get_current_token().unwrap().token_type.clone();
            let right = self.componente()?;
            self._match(op.clone(), true);
            let cursor = self.current_cursor.clone();
            node = TreeNode::new(Node::Exp {
                cursor,
                id: Uuid::new_v4().to_string(),
                typ: ExpType::Void,
                kind: ExpKind::Op {
                    op,
                    left: Box::new(node),
                    right: Some(Box::new(right)),
                },
                val: None,
            });
        }
        Some(node)
    }

    fn componente(&mut self) -> Option<TreeNode> {
        match self.get_current_token().cloned() {
            Some(token) => match token.token_type {
                TokenType::LPAR => {
                    self._match(TokenType::LPAR, true); // siempre es true
                                                        // let cursor = self.current_cursor.clone();
                    let node = self.expresion()?;
                    if !self._match(TokenType::RPAR, true) {
                        // dejamos que quien use la expresion se encargue de definir el punto seguro :)
                        return None;
                    }
                    Some(node)
                }
                TokenType::INT => {
                    let value: i32 = token.lexemme.parse().unwrap();
                    self._match(TokenType::INT, true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    Some(TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        kind: ExpKind::Const { value },
                        typ: ExpType::Void,
                        val: None,
                    }))
                }
                TokenType::FLOAT => {
                    let value: f32 = token.lexemme.parse().unwrap();
                    self._match(TokenType::FLOAT, true); // siempre es true
                    let cursor = self.current_cursor.clone();
                    Some(TreeNode::new(Node::Exp {
                        cursor,
                        id: Uuid::new_v4().to_string(),
                        kind: ExpKind::ConstF { value },
                        typ: ExpType::Void,
                        val: None,
                    }))
                }
                TokenType::ID => self.incremento(),
                _ => {
                    let expected_token_type = vec![TokenType::LPAR, TokenType::INT, TokenType::ID];
                    self.errors.push(ParseError {
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

    fn incremento(&mut self) -> Option<TreeNode> {
        let name = self.get_current_token().unwrap().lexemme.clone();
        self._match(TokenType::ID, true); // siempre es true
        let cursor = self.current_cursor.clone();
        if matches!(
            self.get_current_token().unwrap().token_type,
            TokenType::INC | TokenType::DEC
        ) {
            let op_token = self.get_current_token().unwrap().token_type.clone();
            let op = match op_token {
                TokenType::INC => TokenType::SUM,
                TokenType::DEC => TokenType::MIN,
                _ => TokenType::SUM,
            }; // siempre es true
            self._match(op_token, true);
            let cursor2 = self.current_cursor.clone();

            Some(TreeNode::new(Node::Exp {
                cursor: cursor.clone(),
                id: Uuid::new_v4().to_string(),
                typ: ExpType::Void,
                kind: ExpKind::Op {
                    op,
                    left: Box::new(TreeNode::new(Node::Exp {
                        cursor: cursor2,
                        id: Uuid::new_v4().to_string(),
                        typ: ExpType::Void,
                        kind: ExpKind::Id { name },
                        val: None,
                    })),
                    right: Some(Box::new(TreeNode::new(Node::Exp {
                        cursor: None,
                        id: Uuid::new_v4().to_string(),
                        kind: ExpKind::Const { value: 1 },
                        typ: ExpType::Void,
                        val: None,
                    }))),
                },
                val: None,
            }))
        } else {
            Some(TreeNode::new(Node::Exp {
                cursor,
                id: Uuid::new_v4().to_string(),
                typ: ExpType::Void,
                kind: ExpKind::Id { name },
                val: None,
            }))
        }
    }
}
