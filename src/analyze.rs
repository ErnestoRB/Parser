use std::collections::HashMap;

use scanner::data::TokenType;

use crate::{
    parse::structures::{SymbolData, SymbolReference},
    structures::{DeclKind, ExpKind, Node, NodeValue, StmtKind, SymbolError, TreeNode},
};

pub fn create_symbol_table(node: &TreeNode) -> (HashMap<String, SymbolData>, Vec<SymbolError>) {
    let mut errors: Vec<SymbolError> = vec![];
    let mut location = 0;
    let mut map: HashMap<String, SymbolData> = HashMap::new();
    node.pre_order_traversal(&mut |node: &Node| {
        if let Node::Exp {
            kind, cursor, id, ..
        } = node
        {
            if let crate::structures::ExpKind::Id { name } = kind {
                if !map.contains_key(name) {
                    errors.push(SymbolError {
                        message: "Uso antes de declaración".to_string(),
                        cursor: cursor.clone().unwrap(),
                    });
                } else {
                    match map.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: cursor.clone().unwrap(),
                        }),
                        None => todo!(),
                    };
                }
            }
        }

        if let Node::Decl { kind, cursor, .. } = node {
            if let DeclKind::Var { name, .. } = kind {
                if map.contains_key(name) {
                    errors.push(SymbolError {
                        message: "Doble declaración".to_string(),
                        cursor: cursor.clone().unwrap(),
                    });
                } else {
                    map.insert(
                        name.to_owned(),
                        SymbolData {
                            mem_location: location,
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
                if !map.contains_key(name) {
                    errors.push(SymbolError {
                        message: "Uso antes de declaración".to_string(),
                        cursor: cursor.clone().unwrap(),
                    });
                } else {
                    match map.get_mut(name) {
                        Some(data) => {
                            (*data).usages.push(SymbolReference {
                                cursor: cursor.clone().unwrap(),
                            });
                        }
                        None => todo!(),
                    };
                }
            }
            if let StmtKind::In { name, .. } = kind {
                if !map.contains_key(name) {
                    errors.push(SymbolError {
                        message: "Uso antes de declaración".to_string(),
                        cursor: cursor.clone().unwrap(),
                    });
                } else {
                    match map.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: cursor.clone().unwrap(),
                        }),
                        None => todo!(),
                    };
                }
            }
        }
    });

    (map, errors)
}

pub fn evaluate_arithmetic_expressions(
    node: &mut TreeNode,
    symbol_table: &mut HashMap<String, SymbolData>,
) -> Vec<SymbolError> {
    let mut errors: Vec<SymbolError> = vec![];

    node.post_order_traversal_mut(&mut |node: &mut Node| {
        // Verificamos si es un nodo de expresión

        match node {
            Node::Stmt { kind, .. } => {
                if let StmtKind::Assign { name, value } = kind {
                    // aqui aprovechamos que tenemos el valor calculado (gracias a el recorrido postorden, por lo que guardamos este valor en la tabla de simbolos)
                    match &value.node {
                        Node::Exp { val, .. } => {
                            if let Some(symbol) = symbol_table.get_mut(name) {
                                symbol.value = val.clone();
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
            Node::Exp {
                kind, cursor, val, ..
            } => match kind {
                // Evaluamos expresiones con operadores
                ExpKind::Op { op, left, right } => {
                    // Evaluamos las subexpresiones
                    if let (Some(left_val), Some(right_val)) = (
                        get_expression_value(left, symbol_table),
                        right
                            .as_ref()
                            .and_then(|r| get_expression_value(r, symbol_table)),
                    ) {
                        let result = match op {
                            TokenType::SUM => left_val + right_val,
                            TokenType::MIN => left_val - right_val,
                            TokenType::TIMES => left_val * right_val,
                            TokenType::MODULUS => left_val % right_val,
                            TokenType::POWER => left_val.pow(right_val),
                            TokenType::DIV => {
                                // if right_val == 0 {
                                //     errors.push(SymbolError {
                                //         message: "División por cero".to_string(),
                                //         cursor: cursor.clone().unwrap(),
                                //     });
                                //     return;
                                // }
                                left_val / right_val
                            }
                            _ => {
                                // no deberia pasar
                                panic!("Se asignó un token no valido a la operacion")
                            }
                        };
                        *val = Some(result);
                    } else {
                        *val = None;
                    }
                }
                ExpKind::Id { name } => {
                    // Si es un identificador, comprobamos si está declarado
                    if let Some(symbol) = symbol_table.get(name) {
                    } else {
                        errors.push(SymbolError {
                            message: format!("Variable no declarada: {}", name),
                            cursor: cursor.clone().unwrap(),
                        });
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });

    errors
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
