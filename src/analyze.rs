use std::collections::HashMap;

use scanner::{data::TokenType, utils::fake_cursor};

use crate::{
    parse::structures::{SymbolData, SymbolReference},
    structures::{AnalyzeError, DeclKind, ExpKind, ExpType, Node, NodeValue, StmtKind, TreeNode},
};

pub struct Analyzer {
    pub errors: Vec<AnalyzeError>,
    pub symbol_table: HashMap<String, SymbolData>
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer { errors: vec![], symbol_table: HashMap::new()}
    }
        
    fn create_symbol_table(&mut self,node: &TreeNode) -> () {
        let mut location = 0;
        node.pre_order_traversal(&mut |node: &Node| {
            if let Node::Exp {
                kind, cursor, ..
            } = node
            {
                if let crate::structures::ExpKind::Id { name } = kind {
                    match self.symbol_table.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: cursor.clone().unwrap(),
                        }),
                        None => {
                            self.errors.push(AnalyzeError {
                                message: "Uso antes de declaración".to_string(),
                                cursor: cursor.clone().unwrap(),
                            });
                        },
                    };
                }
            }

            if let Node::Decl { kind, cursor, .. } = node {
                if let DeclKind::Var { typ, name, .. } = kind {
                    if self.symbol_table.contains_key(name) {
                        self.errors.push(AnalyzeError {
                            message: "Doble declaración".to_string(),
                            cursor: cursor.clone().unwrap(),
                        });
                    } else {
                        self.symbol_table.insert(
                            name.to_owned(),
                            SymbolData {
                                mem_location: location,
                                typ: typ.clone(),
                                declaration: cursor.clone().unwrap(),
                                value: None,
                                usages: vec![],
                            },
                        );
                        location += 1;
                    }
                }
            }

            if let Node::Stmt { kind, cursor, .. } = node {
                if let StmtKind::Assign { name, .. } = kind {
                    match self.symbol_table.get_mut(name) {
                        Some(data) => {
                            (*data).usages.push(SymbolReference {
                                cursor: cursor.clone().unwrap(),
                            });
                        }
                        None => {
                            self.errors.push(AnalyzeError {
                                message: "Uso antes de declaración".to_string(),
                                cursor: cursor.clone().unwrap(),
                            });
                        },
                    };
                }
                if let StmtKind::In { name, .. } = kind {
                    match self.symbol_table.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: cursor.clone().unwrap(),
                        }),
                        None => {
                            self.errors.push(AnalyzeError {
                                message: "Uso antes de declaración".to_string(),
                                cursor: cursor.clone().unwrap(),
                            });
                        },
                    };
                }
            }
        });

    }

    fn check_types(&mut self,node: &mut TreeNode
    ) -> () {

        node.post_order_traversal_mut(&mut |node: &mut Node| match node {
            Node::Stmt { kind, ..} => {
                match kind {
                    StmtKind::If { condition, .. } => {
                        if let Node::Exp { typ, cursor: condition_cursor, .. } =  &condition.node {
                            if !matches!(typ, ExpType::Boolean) {
                                self.errors.push(AnalyzeError { message: "Condicion de IF no es booleana".to_string(), cursor: condition_cursor.clone().unwrap_or(fake_cursor()) });
                            }
                        }
                    },
                    StmtKind::While { condition, .. } => {
                        if let Node::Exp { typ, cursor: condition_cursor, .. } =  &condition.node {
                            if !matches!(typ, ExpType::Boolean) {
                                self.errors.push(AnalyzeError { message: "Condicion de While no es booleana".to_string(), cursor: condition_cursor.clone().unwrap_or(fake_cursor()) });
                            }
                        }
                    },
                    StmtKind::Do { condition,.. } =>  if let Node::Exp { typ, cursor: condition_cursor, .. } =  &condition.node {
                        if !matches!(typ, ExpType::Boolean) {
                            self.errors.push(AnalyzeError { message: "Condicion de Do no es booleana".to_string(), cursor: condition_cursor.clone().unwrap_or(fake_cursor()) });
                        }
                    },
                    StmtKind::Assign { value, name } => 
                    if let Node::Exp { typ, cursor: value_cursor, .. } =  &value.node {
                        if let Some(symbol) = self.symbol_table.get(name) {

                            if matches!(typ, ExpType::Boolean) {
                                self.errors.push(AnalyzeError { message: "No se puede asignar una expresion booleana a una variable".to_string(), cursor: value_cursor.clone().unwrap_or(fake_cursor()) });
                            } else {
                                if &symbol.typ != typ {
                                    self.errors.push(AnalyzeError { message: "Se asigno un valor de tipo distinto!".to_string(), cursor: value_cursor.clone().unwrap_or(fake_cursor()) });
                                }
                            }
                        }
                        
                    }
                    ,
                    StmtKind::In {.. } => {},
                    StmtKind::Out {expression } => {
                        if let Node::Exp { typ, cursor: exp_cursor, ..} = &expression.node  {
                            if !matches!(typ, ExpType::Integer | ExpType::Float) {
                                self.errors.push(AnalyzeError { message: "La sentencia out solo está definida para expresiones Enteras y Flotantes!".to_string(), cursor:  exp_cursor.clone().unwrap_or(fake_cursor()) });
                            }
                        }
                    }
                }

            },
            Node::Exp {
                kind, typ, cursor, ..
            } => match kind {
                ExpKind::Op { op, left, right } => {
                match right {
                        Some(right_node) => {
                            match op {
                                TokenType::SUM
                                | TokenType::MIN
                                | TokenType::TIMES
                                | TokenType::DIV
                                | TokenType::MODULUS
                                | TokenType::POWER => {
                                    if let (
                                        Node::Exp { typ: l_type, .. },
                                        Node::Exp { typ: r_type, .. },
                                    ) = (&left.node, &right_node.node)
                                    {
                                    // println!("{:?},{:?},{:?}", op, l_type, r_type);
                                        if (!matches!(l_type, ExpType::Integer)
                                            && !matches!(l_type, ExpType::Float)) || (!matches!(r_type, ExpType::Integer)
                                            && !matches!(r_type, ExpType::Float))
                                        {
                                            self.errors.push(AnalyzeError {
                                                message: "Una operacion arimetica no puede ser aplicada en un tipo distinto a float o entero"
                                                    .to_string(),
                                                cursor: cursor.clone().unwrap_or(fake_cursor()),
                                                
                                            });
                                        }
                                        if matches!(l_type, ExpType::Float) || matches!(r_type, ExpType::Float) {
                                            // inferencia de tipo
                                            *typ = ExpType::Float;
                                        } else {
                                            *typ = ExpType::Integer;
                                        }
                                    }
                                }
                                TokenType::LT
                                | TokenType::LE
                                | TokenType::GT
                                | TokenType::GE
                                => 
                                {
                                    if let (
                                        Node::Exp { typ: l_type, .. },
                                        Node::Exp { typ: r_type, .. },
                                    ) = (&left.node, &right_node.node)
                                    {
                                        if (!matches!(l_type, ExpType::Integer)
                                            && !matches!(l_type, ExpType::Float)) || (!matches!(r_type, ExpType::Integer)
                                            && !matches!(r_type, ExpType::Float))
                                        {
                                            self.errors.push(AnalyzeError {
                                                message: "Una operacion de comparacion aritmetica no puede ser aplicada en un tipo distinto a float o entero"
                                                    .to_string(),
                                                cursor: cursor.clone().unwrap_or(fake_cursor()),
                                            });
                                        }
                                        // inferencia de tipos
                                        *typ = ExpType::Boolean;
                                    }
                                },
                                TokenType::NE
                                | TokenType::EQ => {
                                    if let (
                                        Node::Exp { typ: l_type, .. },
                                        Node::Exp { typ: r_type, .. },
                                    ) = (&left.node, &right_node.node)
                                    {
                                        if (!matches!(l_type, ExpType::Integer)
                                            && !matches!(l_type, ExpType::Float) && !matches!(l_type, ExpType::Boolean)) || (!matches!(r_type, ExpType::Integer)
                                            && !matches!(r_type, ExpType::Float) && !matches!(r_type, ExpType::Boolean))
                                        {
                                            self.errors.push(AnalyzeError {
                                                message: "Una operacion de comparacion aritmetica no puede ser aplicada en un tipo distinto a float o entero"
                                                    .to_string(),
                                                cursor: cursor.clone().unwrap_or(fake_cursor()),
                                            });
                                        }
                                        // inferencia de tipos
                                        *typ = ExpType::Boolean;
                                    }
                                }
                                ,
                                TokenType::AND | TokenType::OR => {
                                    if let (
                                        Node::Exp { typ: l_type, .. },
                                        Node::Exp { typ: r_type, .. },
                                    ) = (&left.node, &right_node.node)
                                    {
                                        if !matches!(l_type, ExpType::Boolean) ||  !matches!(r_type, ExpType::Boolean)
                                        {
                                            self.errors.push(AnalyzeError {
                                                message: "Una operacion lógica no puede ser aplicada en un tipo distinto a booleano"
                                                    .to_string(),
                                                cursor: cursor.clone().unwrap_or(fake_cursor()),
                                            });
                                        }
                                        // inferencia de tipos
                                        *typ = ExpType::Boolean;
                                    }
                                },
                                _ => panic!("Operacion incorrecta. Bug en el analizador sintactico"),
                            };
        

                        },
                        None => {
                            if !matches!(op, TokenType::NEG) {
                                self.errors.push(AnalyzeError {
                                    message: "Operacion binaria sin lado derecho".to_string(),
                                    cursor: cursor.clone().unwrap_or(fake_cursor()),
                                });
                            } else {
                                // inferencia
                                *typ = ExpType::Boolean;
                                if let Node::Exp {  typ, ..} = &left.node {
                                    if !matches!(typ, ExpType::Boolean)  {
                                        self.errors.push(AnalyzeError {
                                            message: "Operacion de negacion solo esta disponible para booleanos".to_string(),
                                            cursor: cursor.clone().unwrap_or(fake_cursor()),
                                        });
                                    }
                                }
                            }
                        },
                    }

                
                
                }
                ExpKind::Const { .. } => *typ = ExpType::Integer,
                ExpKind::ConstF { .. } => *typ = ExpType::Float,
                ExpKind::Id { name } => {
                    if let Some(symbol) = self.symbol_table.get(name) {
                        *typ = symbol.typ.clone();
                    }
                }
            },
            Node::Decl { .. } => {},
        });

    }

    fn evaluate_expressions(
        &mut self,
        node: &mut TreeNode,
    ) -> (){
        node.post_order_traversal_mut(&mut |node: &mut Node| {
            // Verificamos si es un nodo de expresión
            match node {
                Node::Stmt { kind, .. } => {
                    if let StmtKind::Assign { name, value } = kind {
                        // aqui aprovechamos que tenemos el valor calculado (gracias a el recorrido postorden, por lo que guardamos este valor en la tabla de simbolos)
                        match &value.node {
                            Node::Exp { val, .. } => {
                                if let Some(symbol) = self.symbol_table.get_mut(name) {
                                    symbol.value = val.clone();
                                }
                            }
                            _ => panic!("Error en el analizador sintáctico. El lado derecho de una asignación no fue expresión."),
                        }
                    }
                }
                Node::Exp {
                    kind, cursor, val, ..
                } => match kind {
                    // Evaluamos expresiones con operadores
                    ExpKind::Op { op, left, right } => {
                        // Evaluamos las subexpresiones (si hay valor)
                    
                        if let Some(left_val) = 
                            get_expression_value(left, &self.symbol_table)
                        {

                            match  right
                            .as_ref()
                            .and_then(|r| get_expression_value(r, &self.symbol_table)) {
                                Some(right_val) => {
                                    // if matches!(typ, ExpType::Float) {
                                        //     left_val  = left_val.to_float();
                                        //     right_val = right_val.to_float();
                                        // }
                                    let result = match op {
                                        TokenType::SUM => {
                                            let op = left_val + right_val;
                                            if let None = &op {
                                                self.errors.push(AnalyzeError { message: 
                                                    "La suma solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                            }
                                            op
                                        },
                                        TokenType::MIN => {
                                            let op = left_val - right_val;
                                            if let None = &op {
                                                self.errors.push(AnalyzeError { message: 
                                                    "La resta solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                            }
                                            op
                                        },
                                        TokenType::TIMES => {
                                                let op = left_val * right_val;
                                                if let None = &op {
                                                    self.errors.push(AnalyzeError { message: 
                                                        "La multiplicación solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                                }
                                                op
                                        },
                                        TokenType::MODULUS => {
                                            let op = left_val % right_val;
                                            if let None = &op {
                                                self.errors.push(AnalyzeError { message: 
                                                    "El módulo solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                            }
                                            op
                                        },
                                        TokenType::POWER => {
                                            let op = left_val.pow(right_val);
                                            if let None = &op {
                                                self.errors.push(AnalyzeError { message: 
                                                    "La exponenciación solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                            }
                                            op
                                        },                                     
                                        TokenType::DIV => {
                                            {
                                                match right_val {
                                                    NodeValue::Int(rv) if rv == 0 => {
                                                        self.errors.push(AnalyzeError {
                                                            message: "División entre cero".to_string(),
                                                            cursor: cursor.clone().unwrap(),
                                                        });
                                                        return;
                                                    }
                                                    NodeValue::Float(rv) if rv == 0.0 => {
                                                        self.errors.push(AnalyzeError {
                                                            message: "División entre cero".to_string(),
                                                            cursor: cursor.clone().unwrap(),
                                                        });
                                                        return;
                                                    }
                                                    _ => {}
                                                }
                                                let op = left_val / right_val;
                                                if let None = &op {
                                                    self.errors.push(AnalyzeError { message: 
                                                        "La división solo está definida para numeros".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                                }
                                                op
                                            }
                                        }
                                        TokenType::NE => Some({
                                            NodeValue::Boolean(left_val != right_val)
                                        }),
                                        TokenType::EQ => Some({
                                            NodeValue::Boolean(left_val == right_val)
                                        }),
                                        TokenType::LT => Some({
                                            NodeValue::Boolean(left_val < right_val)
                                        }),
                                        TokenType::GT => Some({
                                            NodeValue::Boolean(left_val > right_val)
                                        }),
                                        TokenType::GE => Some({
                                            NodeValue::Boolean(left_val >= right_val)
                                        }),
                                        TokenType::LE => Some({
                                            NodeValue::Boolean(left_val <= right_val)
                                        }),
                                        TokenType::AND => {
                                            if let (NodeValue::Boolean(l), NodeValue::Boolean(r)) = (left_val, right_val) {
                                                Some(NodeValue::Boolean(l && r))
                                            } else {
                                                self.errors.push(AnalyzeError { message: 
                                                    "El operador AND solo está definida para booleanos".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                                None
                                            }
                                        },
                                        TokenType::OR => {
                                            if let (NodeValue::Boolean(l), NodeValue::Boolean(r)) = (left_val, right_val) {
                                               Some( NodeValue::Boolean(l || r))
                                            } else {
                                                self.errors.push(AnalyzeError { message: 
                                                    "El operador OR solo está definida para booleanos".to_string(), cursor:  cursor.clone().unwrap_or(fake_cursor()) });
                                                None
                                            }
                                        }
                                        _ => {
                                            // no deberia pasar
                                            panic!("Se asignó un token no valido a la operacion")
                                        }
                                    };
                                    *val = result;
                                },
                                None => {
                                    if matches!(op, TokenType::NEG) {
                                        if let NodeValue::Boolean(b) = left_val {
                                            *val = Some(NodeValue::Boolean(!b));
                                        } else {
                                            // errors.push("value");
                                        }
                                    } else {

                                    }
                                }, // NEGACION (unica operacion binaria)
                            }
                        
                        
                        } else { // aquí, al no tener algun valor en izq o derecha, se da valor ausente
                            *val = None;
                        }
                    }
                    ExpKind::Id { name } => {
                        // Si es un identificador, comprobamos si está declarado
                        if let Some(symbol) = self.symbol_table.get(name) {
                            *val = symbol.value.clone(); // obtenemos el valor de la tabla de símbolos :)
                        } else {
                            self.errors.push(AnalyzeError {
                                message: format!("No se puede evaluar el valor de una variable no declarada: {}", name),
                                cursor: cursor.clone().unwrap(),
                            });
                        }

                    },
                    ExpKind::Const { value } => *val = Some(NodeValue::Int(value.clone())),
                    ExpKind::ConstF { value, } => *val = Some(NodeValue::Float(value.clone())),
                    _ => {}
                },
                _ => {}
            }
        });

    }

    pub fn analyze(mut self, node: &mut TreeNode) -> (Vec<AnalyzeError>,HashMap<String, SymbolData>) {
        self.create_symbol_table(node);
        self.check_types(node);
        self.evaluate_expressions(node);
        (self.errors, self.symbol_table)
    }
}


pub fn analyze(node: &mut TreeNode) -> (Vec<AnalyzeError>,HashMap<String, SymbolData>) {
    let analyzer = Analyzer::new();
    analyzer.analyze(node)
}

fn get_expression_value(
    node: &TreeNode,
    symbol_table: &HashMap<String, SymbolData>,
) -> Option<NodeValue> {
    if let Node::Exp { kind, val, .. } = &node.node {
        match kind {
            ExpKind::Const { value } => Some(NodeValue::Int(*value)), // Constantes enteras
            ExpKind::ConstF { value } => Some(NodeValue::Float(*value)), // Constantes enteras
            ExpKind::Id { name } => {
                // Si es un identificador, buscamos su valor en la tabla de símbolos
                symbol_table.get(name).map(|data| data.value.clone())? // Devuelve el valor asignado a la variable como valor
            }
            ExpKind::Op { .. } => val.clone(),
        }
    } else {
        None
    }
}


pub fn debug(node: &TreeNode) {
    node.pre_order_traversal(&mut |node| {
        if let Node::Exp { kind, .. } = node {
            if let ExpKind::Id { name } = kind {
                println!("{:?}\n-------", node)
            }
        }
    });
}
