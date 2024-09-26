use std::collections::HashMap;

use super::structures::{DeclKind, ExpKind, Node, StmtKind, SymbolData, TreeNode};

impl TreeNode {
    pub fn new(node: Node) -> Self {
        TreeNode {
            node,
            //children: Vec::new(),
            sibling: None,
        }
    }

    /*   pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(Box::new(child));
    } */

    // Función para recorrer el árbol de manera preorden
    pub fn pre_order_traversal(&self, visit: &mut dyn FnMut(&Node)) {
        // Primero visitamos el nodo actual
        visit(&self.node);

        // Dependiendo del tipo de nodo, recorremos sus "hijos" implícitos
        match &self.node {
            // Si el nodo es una sentencia (Stmt), verificamos su tipo
            Node::Stmt { kind, .. } => match kind {
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    // Visitamos la condición (Exp)
                    visit(condition);
                    // Si existe el bloque "then", lo recorremos
                    if let Some(then) = then_branch {
                        then.pre_order_traversal(visit);
                    }
                    // Si existe el bloque "else", lo recorremos
                    if let Some(else_) = else_branch {
                        else_.pre_order_traversal(visit);
                    }
                }
                StmtKind::While { condition, body } => {
                    // Visitamos la condición (Exp)
                    visit(condition);
                    // Si existe el cuerpo, lo recorremos
                    if let Some(body) = body {
                        body.pre_order_traversal(visit);
                    }
                }
                StmtKind::Do { body, condition } => {
                    // Visitamos el cuerpo si existe
                    if let Some(body) = body {
                        body.pre_order_traversal(visit);
                    }
                    // Luego visitamos la condición (Exp)
                    visit(condition);
                }
                StmtKind::Assign { value, .. } => {
                    // Visitamos el valor de la asignación (Exp)
                    visit(value);
                }
                StmtKind::In { .. } | StmtKind::Out { .. } => {
                    // Si es un Stmt de entrada o salida, no tiene hijos
                }
            },
            // Si el nodo es una expresión (Exp), verificamos su tipo
            Node::Exp { kind, .. } => match kind {
                ExpKind::Op { left, right, .. } => {
                    // Visitamos el lado izquierdo (siempre existe)
                    visit(left);
                    // Si hay un lado derecho, también lo visitamos
                    if let Some(right) = right {
                        visit(right);
                    }
                }
                ExpKind::Const { .. } | ExpKind::ConstF { .. } | ExpKind::Id { .. } => {
                    // No hay hijos en estos casos
                }
            },
            // Si el nodo es una declaración (Decl), no tiene hijos implícitos
            Node::Decl { .. } => {}
        }

        // Finalmente, recorremos los hermanos si existen
        if let Some(sibling) = &self.sibling {
            sibling.pre_order_traversal(visit);
        }
    }

    pub fn print(&self) {
        print_tree(self, 0);
    }
    pub fn last_sibling(&self) -> Option<&Box<TreeNode>> {
        let mut current = self.sibling.as_ref();
        while let Some(ref next) = current {
            if next.sibling.is_none() {
                break;
            }
            current = next.sibling.as_ref();
        }
        current
    }
    pub fn get_last_sibling_mut(&mut self) -> &mut TreeNode {
        // Primero, verificamos si hay un hermano
        if let Some(ref mut sibling) = self.sibling {
            // Si hay un hermano, llamamos recursivamente a la función en ese hermano
            sibling.get_last_sibling_mut()
        } else {
            // Si no hay un hermano, retornamos una referencia mutable al nodo actual
            self
        }
    }
}

fn print_tree(node: &TreeNode, indent: usize) {
    let indentation = " ".repeat(indent);
    match &node.node {
        Node::Decl { kind, id: _ } => match kind {
            DeclKind::Var { typ, name } => {
                println!("{}Decl: Var (Type: {:?}, Name: {})", indentation, typ, name);
            }
        },
        Node::Stmt { kind, id: _ } => match kind {
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                println!("{}Stmt: If", indentation);
                println!("{}  Condition:", indentation);
                print_tree_node(condition, indent + 4);
                println!("{}  Then Branch:", indentation);
                if let Some(then) = then_branch {
                    print_tree(then, indent + 4);
                }
                if let Some(else_branch) = else_branch {
                    println!("{}  Else Branch:", indentation);
                    print_tree(else_branch, indent + 4);
                }
            }
            StmtKind::While { condition, body } => {
                println!("{}Stmt: While", indentation);
                println!("{}  Condition:", indentation);
                print_tree_node(condition, indent + 4);
                println!("{}  Body:", indentation);
                if let Some(body) = body {
                    print_tree(body, indent + 4);
                }
            }
            StmtKind::Do { body, condition } => {
                println!("{}Stmt: Do", indentation);
                println!("{}  Body:", indentation);
                if let Some(body) = body {
                    print_tree(body, indent + 4);
                }
                println!("{}  Condition:", indentation);
                print_tree_node(condition, indent + 4);
            }
            StmtKind::Assign { name, value } => {
                println!("{}Stmt: Assign (Name: {})", indentation, name);
                println!("{}  Value:", indentation);
                print_tree_node(value, indent + 4);
            }
            StmtKind::In { name } => {
                println!("{}Stmt: In (Name: {})", indentation, name);
            }
            StmtKind::Out { expression } => {
                println!("{}Stmt: Out", indentation);
                println!("{}  Expression:", indentation);
                print_tree_node(expression, indent + 4);
            }
        },
        Node::Exp { kind, typ, id: _ } => match kind {
            ExpKind::Op { op, left, right } => {
                println!("{}Exp: Op ({:?})", indentation, op);
                println!("{}  Left:", indentation);
                print_tree_node(left, indent + 4);
                println!("{}  Right:", indentation);
                if let Some(right_node) = right {
                    print_tree_node(&right_node, indent + 4);
                }
            }
            ExpKind::Const { value } => {
                println!("{}Exp: Const (Value: {})", indentation, value);
            }
            ExpKind::ConstF { value } => {
                println!("{}Exp: Const Float (Value: {})", indentation, value);
            }
            ExpKind::Id { name } => {
                println!("{}Exp: Id (Name: {})", indentation, name);
            }
        },
    }
    /*  for child in &node.children {
        print_tree(child, indent + 2);
    } */
    if let Some(sibling) = &node.sibling {
        print_tree(sibling, indent);
    }
}

fn print_tree_node(node: &Node, indent: usize) {
    let temp_node = TreeNode {
        node: node.clone(),
        // children: vec![],
        sibling: None,
    };
    print_tree(&temp_node, indent);
}

pub fn print_sym_table(table: HashMap<String, SymbolData>) {
    for (k, v) in table.iter() {
        print!(
            "Variable:  {}  - ({},{}) | Location {} | Usages: ",
            k, v.declaration.col, v.declaration.lin, v.mem_location
        );
        for usage in v.usages.iter() {
            print!("({}, {}),", usage.cursor.col, usage.cursor.lin)
        }
        println!();
    }
}
