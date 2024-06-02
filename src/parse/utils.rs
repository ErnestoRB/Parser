use super::structures::{DeclKind, ExpKind, Node, StmtKind, TreeNode};

impl TreeNode {
    pub fn new(node: Node) -> Self {
        TreeNode {
            node,
            children: Vec::new(),
            sibling: None,
        }
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(Box::new(child));
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
        Node::Decl(decl) => match decl {
            DeclKind::Var { typ, name } => {
                println!("{}Decl: Var (Type: {:?}, Name: {})", indentation, typ, name);
            }
        },
        Node::Stmt(stmt) => match stmt {
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                println!("{}Stmt: If", indentation);
                println!("{}  Condition:", indentation);
                print_tree_node(condition, indent + 4);
                println!("{}  Then Branch:", indentation);
                print_tree(then_branch, indent + 4);
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
                print_tree(body, indent + 4);
            }
            StmtKind::Do { body, condition } => {
                println!("{}Stmt: Do", indentation);
                println!("{}  Body:", indentation);
                print_tree(body, indent + 4);
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
        Node::Exp { kind, typ } => match kind {
            ExpKind::Op { op, left, right } => {
                println!("{}Exp: Op ({:?})", indentation, op);
                println!("{}  Left:", indentation);
                print_tree_node(left, indent + 4);
                println!("{}  Right:", indentation);
                print_tree_node(right, indent + 4);
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
    for child in &node.children {
        print_tree(child, indent + 2);
    }
    if let Some(sibling) = &node.sibling {
        print_tree(sibling, indent);
    }
}

fn print_tree_node(node: &Node, indent: usize) {
    let temp_node = TreeNode {
        node: node.clone(),
        children: vec![],
        sibling: None,
    };
    print_tree(&temp_node, indent);
}
