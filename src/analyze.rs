use std::collections::HashMap;

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
                println!("{}", name);
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
