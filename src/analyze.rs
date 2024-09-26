use std::collections::HashMap;

use scanner::utils::fake_cursor;

use crate::{
    parse::structures::{SymbolData, SymbolReference},
    structures::{DeclKind, Node, StmtKind, TreeNode},
};

pub fn create_symbol_table(node: &TreeNode) -> (HashMap<String, SymbolData>, ()) {
    let mut location = 0;
    let mut map: HashMap<String, SymbolData> = HashMap::new();
    node.pre_order_traversal(&mut |node: &Node| {
        if let Node::Exp { kind, id, .. } = node {
            println!("{}", id);
            if let crate::structures::ExpKind::Id { name } = kind {
                if !map.contains_key(name) {
                    todo!("Error de uso antes de declaración")
                } else {
                    match map.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: fake_cursor(),
                        }),
                        None => todo!(),
                    };
                }
            }
        }

        if let Node::Decl { kind, .. } = node {
            if let DeclKind::Var { name, .. } = kind {
                if map.contains_key(name) {
                    todo!("Error de doble declaración")
                } else {
                    map.insert(
                        name.to_owned(),
                        SymbolData {
                            mem_location: location,
                            declaration: fake_cursor(),
                            usages: vec![],
                        },
                    );
                    location += 1;
                }
            }
        }

        if let Node::Stmt { kind, .. } = node {
            if let StmtKind::Assign { name, .. } = kind {
                if !map.contains_key(name) {
                    todo!("Error de uso antes de declaración")
                } else {
                    match map.get_mut(name) {
                        Some(data) => (*data).usages.push(SymbolReference {
                            cursor: fake_cursor(),
                        }),
                        None => todo!(),
                    };
                }
            }
        }
    });

    (map, ())
}
