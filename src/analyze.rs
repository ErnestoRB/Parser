use std::collections::HashMap;

use scanner::data::TokenType;

use crate::{
    parse::structures::{SymbolData, SymbolReference},
    structures::{DeclKind, ExpKind, Node, StmtKind, SymbolError, TreeNode},
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
                            usages: vec![],
                            value: None,
                        },
                    );
                    location += 1;
                }
            }
        }

        if let Node::Stmt { kind, cursor, .. } = node {
            if let StmtKind::Assign { name, value, .. } = kind {
                if !map.contains_key(name) {
                    errors.push(SymbolError {
                        message: "Uso antes de declaración".to_string(),
                        cursor: cursor.clone().unwrap(),
                    });
                } else {
                    let eval_result = evaluate_expression(value, &map);
                    match eval_result {
                        Ok(val) => {
                            if let Some(data) = map.get_mut(name) {
                                data.value = Some(val);
                                data.usages.push(SymbolReference {
                                    cursor: cursor.clone().unwrap(),
                                });
                            }
                        }
                        Err(err) => {
                            errors.push(SymbolError {
                                message: err,
                                cursor: cursor.clone().unwrap(),
                            });
                        }
                    }
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

pub fn evaluate_expression(
    node: &TreeNode,
    symbol_table: &HashMap<String, SymbolData>,
) -> Result<i32, String> {
    match &node.node {
        Node::Exp { kind, .. } => match kind {
            ExpKind::Const { value } => Ok(*value),
            ExpKind::Id { name } => {
                if let Some(symbol_data) = symbol_table.get(name) {
                    if let Some(value) = symbol_data.value {
                        Ok(value)
                    } else {
                        Err(format!("Variable '{}' no tiene un valor asignado", name))
                    }
                } else {
                    Err(format!("Variable '{}' no está declarada", name))
                }
            }
            ExpKind::Op { op, left, right } => {
                let left_val = evaluate_expression(left, symbol_table)?;
                let right_val = if let Some(right_node) = right {
                    evaluate_expression(right_node, symbol_table)?
                } else {
                    0
                };

                match op {
                    TokenType::SUM => Ok(left_val + right_val),
                    TokenType::MIN => Ok(left_val - right_val),
                    TokenType::MODULUS => Ok(left_val % right_val),
                    TokenType::POWER => Ok(left_val.pow(right_val as u32)),
                    TokenType::TIMES => Ok(left_val * right_val),
                    TokenType::DIV => {
                        if right_val != 0 {
                            Ok(left_val / right_val)
                        } else {
                            Err("Error: División por cero".to_string())
                        }
                    }
                    _ => Err(format!("Operador '{:?}' no soportado", op)),
                }
            }
            _ => Err("Expresión no soportada para evaluación".to_string()),
        },
        _ => Err("Nodo no es una expresión".to_string()),
    }
}

pub fn debug(node: &TreeNode) {
    node.pre_order_traversal(&mut |node| {
        if let Node::Exp {
            kind,
            typ,
            id,
            cursor,
        } = node
        {
            if let ExpKind::Id { name } = kind {
                println!("{:?}\n-------", node)
            }
        }
    });
}
