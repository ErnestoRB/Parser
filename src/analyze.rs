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
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: cursor.clone().unwrap(),
                        }),
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
    node: &TreeNode,
    symbol_table: &mut HashMap<String, SymbolData>,
) -> Vec<SymbolError> {
    let mut errors: Vec<SymbolError> = vec![];

    node.post_order_traversal(&mut |node: &Node| {
        // Verificamos si es un nodo de expresión
        if let Node::Exp { kind, cursor, .. } = node {
            match kind {
                // Evaluamos expresiones con operadores
                ExpKind::Op { op, left, right } => {
                    // Evaluamos las subexpresiones
                    if let (Some(left_val), Some(right_val)) = (
                        evaluate_expression(left, symbol_table),
                        right
                            .as_ref()
                            .and_then(|r| evaluate_expression(r, symbol_table)),
                    ) {
                        let result = match op {
                            TokenType::SUM => left_val + right_val,
                            TokenType::MIN => left_val - right_val,
                            TokenType::TIMES => left_val * right_val,
                            TokenType::MODULUS => left_val % right_val,
                            TokenType::POWER => left_val.pow(right_val as u32),
                            TokenType::DIV => {
                                if right_val == 0 {
                                    errors.push(SymbolError {
                                        message: "División por cero".to_string(),
                                        cursor: cursor.clone().unwrap(),
                                    });
                                    return;
                                }
                                left_val / right_val
                            }
                            _ => {
                                0
                            }
                        };
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
            }
        }

        // // Evaluamos las sentencias
        // if let Node::Stmt { kind, cursor, .. } = node {
        //     match kind {
        //         StmtKind::Assign { name, value } => {
        //             if let Some(eval_value) = evaluate_expression(value, symbol_table) {
        //                 // Guardamos el valor evaluado en la tabla de símbolos
        //                 if let Some(symbol) = symbol_table.get_mut(name) {
        //                     symbol.mem_location = eval_value;
        //                 } else {
        //                     errors.push(SymbolError {
        //                         message: format!("Asignación a variable no declarada: {}", name),
        //                         cursor: cursor.clone().unwrap(),
        //                     });
        //                 }
        //             }
        //         }
        //         StmtKind::In { name } => {
        //             if symbol_table.get(name).is_none() {
        //                 errors.push(SymbolError {
        //                     message: format!("Entrada de variable no declarada: {}", name),
        //                     cursor: cursor.clone().unwrap(),
        //                 });
        //             }
        //         }
        //         StmtKind::Out { expression } => {
        //             if let Some(expr_value) = evaluate_expression(expression, symbol_table) {
        //                 // Expresión evaluada correctamente, no hace falta hacer nada especial
        //             } else {
        //                 // Si hay una variable no declarada en la expresión de salida
        //                 if let Node::Exp {
        //                     kind: ExpKind::Id { name },
        //                     cursor,
        //                     ..
        //                 } = &expression.node
        //                 {
        //                     if symbol_table.get(name).is_none() {
        //                         errors.push(SymbolError {
        //                             message: format!("Salida con variable no declarada: {}", name),
        //                             cursor: cursor.clone().unwrap(),
        //                         });
        //                     }
        //                 }
        //             }
        //         }
        //         StmtKind::If {
        //             condition,
        //             then_branch,
        //             else_branch,
        //         } => {
        //             if let Some(cond_value) = evaluate_expression(condition, symbol_table) {
        //                 // Condición evaluada correctamente
        //             } else {
        //                 // Verificar si la variable usada en la condición está declarada
        //                 if let Node::Exp {
        //                     kind: ExpKind::Id { name },
        //                     cursor,
        //                     ..
        //                 } = &condition.node
        //                 {
        //                     if symbol_table.get(name).is_none() {
        //                         errors.push(SymbolError {
        //                             message: format!("Variable no declarada: {}", name),
        //                             cursor: cursor.clone().unwrap(),
        //                         });
        //                     }
        //                 }
        //             }
        //             if let Some(then_node) = then_branch {
        //                 evaluate_arithmetic_expressions(then_node, symbol_table);
        //                 // Verificar variables usadas dentro de la rama 'then'
        //                 if let Node::Stmt { kind, cursor, .. } = &then_node.node {
        //                     if let StmtKind::Assign { name, .. } = kind {
        //                         if symbol_table.get(name).is_none() {
        //                             errors.push(SymbolError {
        //                                 message: format!(
        //                                     "Variable no declarada en la rama then: {}",
        //                                     name
        //                                 ),
        //                                 cursor: cursor.clone().unwrap(),
        //                             });
        //                         }
        //                     }
        //                 }
        //             }
        //             // Evaluar la rama 'else'
        //             if let Some(else_node) = else_branch {
        //                 evaluate_arithmetic_expressions(else_node, symbol_table);
                
        //                 // Verificar variables usadas dentro de la rama 'else'
        //                 if let Node::Stmt { kind, cursor, .. } = &else_node.node {
        //                     if let StmtKind::Assign { name, .. } = kind {
        //                         if symbol_table.get(name).is_none() {
        //                             errors.push(SymbolError {
        //                                 message: format!("Variable no declarada en la rama else: {}", name),
        //                                 cursor: cursor.clone().unwrap(),
        //                             });
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
    });

    errors
}

fn evaluate_expression(node: &TreeNode, symbol_table: &HashMap<String, SymbolData>) -> Option<i32> {
    if let Node::Exp { kind, .. } = &node.node {
        match kind {
            ExpKind::Const { value } => Some(*value), // Constantes enteras
            ExpKind::Id { name } => {
                // Si es un identificador, buscamos su valor en la tabla de símbolos
                symbol_table.get(name).map(|data| data.mem_location) // Devuelve la ubicación en memoria como valor
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn debug(node: &TreeNode) {
    node.pre_order_traversal(&mut |node| {
        if let Node::Exp {
            kind,
            typ,
            id,
            cursor,
            val,
        } = node
        {
            if let ExpKind::Id { name } = kind {
                println!("{:?}\n-------", node)
            }
        }
    });
}
