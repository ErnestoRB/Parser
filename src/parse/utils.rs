use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Rem, Sub},
};

use crate::structures::NodeValue;

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

        // Dependiendo del tipo de nodo, recorremos sus hijos
        match &self.node {
            // Si el nodo es una sentencia (Stmt), verificamos su tipo
            Node::Stmt { kind, .. } => match kind {
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.pre_order_traversal(visit);
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
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.pre_order_traversal(visit);
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
                    // Luego visitamos la condición (que es un Node) directamente
                    condition.pre_order_traversal(visit);
                }
                StmtKind::Assign { value, .. } => {
                    // Visitamos el valor de la asignación (que es un TreeNode) de manera recursiva
                    value.pre_order_traversal(visit);
                }
                StmtKind::In { .. } => {
                    // No tiene hijos
                }
                StmtKind::Out { expression } => {
                    // Para el caso de "Out", visitamos la expresión
                    expression.pre_order_traversal(visit);
                }
            },
            // Si el nodo es una expresión (Exp), verificamos su tipo
            Node::Exp { kind, .. } => match kind {
                ExpKind::Op { left, right, .. } => {
                    // Visitamos el lado izquierdo de manera recursiva
                    left.pre_order_traversal(visit);
                    // Si hay un lado derecho, también lo visitamos de manera recursiva
                    if let Some(right) = right {
                        right.pre_order_traversal(visit);
                    }
                }
                ExpKind::Const { .. } | ExpKind::ConstF { .. } | ExpKind::Id { .. } => {
                    // Estos nodos no tienen hijos
                }
            },
            // Si el nodo es una declaración (Decl), verificamos el tipo
            Node::Decl { kind, .. } => match kind {
                DeclKind::Var { .. } => {
                    // No hay hijos en este caso
                }
            },
        }

        // Finalmente, recorremos los hermanos si existen
        if let Some(sibling) = &self.sibling {
            sibling.pre_order_traversal(visit);
        }
    }

    pub fn post_order_traversal(&self, visit: &mut dyn FnMut(&Node)) {
        // Primero recorremos los hijos dependiendo del tipo de nodo
        match &self.node {
            // Si el nodo es una sentencia (Stmt), verificamos su tipo
            Node::Stmt { kind, .. } => match kind {
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.post_order_traversal(visit);
                    // Si existe el bloque "then", lo recorremos
                    if let Some(then) = then_branch {
                        then.post_order_traversal(visit);
                    }
                    // Si existe el bloque "else", lo recorremos
                    if let Some(else_) = else_branch {
                        else_.post_order_traversal(visit);
                    }
                }
                StmtKind::While { condition, body } => {
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.post_order_traversal(visit);
                    // Si existe el cuerpo, lo recorremos
                    if let Some(body) = body {
                        body.post_order_traversal(visit);
                    }
                }
                StmtKind::Do { body, condition } => {
                    // Visitamos el cuerpo si existe
                    if let Some(body) = body {
                        body.post_order_traversal(visit);
                    }
                    // Luego visitamos la condición (que es un Node) directamente
                    condition.post_order_traversal(visit);
                }
                StmtKind::Assign { value, .. } => {
                    // Visitamos el valor de la asignación (que es un TreeNode) de manera recursiva
                    value.post_order_traversal(visit);
                }
                StmtKind::In { .. } => {
                    // No tiene hijos
                }
                StmtKind::Out { expression } => {
                    // Para el caso de "Out", visitamos la expresión
                    expression.post_order_traversal(visit);
                }
            },
            // Si el nodo es una expresión (Exp), verificamos su tipo
            Node::Exp { kind, .. } => match kind {
                ExpKind::Op { left, right, .. } => {
                    // Visitamos el lado izquierdo de manera recursiva
                    left.post_order_traversal(visit);
                    // Si hay un lado derecho, también lo visitamos de manera recursiva
                    if let Some(right) = right {
                        right.post_order_traversal(visit);
                    }
                }
                ExpKind::Const { .. } | ExpKind::ConstF { .. } | ExpKind::Id { .. } => {
                    // Estos nodos no tienen hijos
                }
            },
            // Si el nodo es una declaración (Decl), verificamos el tipo
            Node::Decl { kind, .. } => match kind {
                DeclKind::Var { .. } => {
                    // No hay hijos en este caso
                }
            },
        }

        // Finalmente, visitamos el nodo actual
        visit(&self.node);

        // Luego, recorremos los hermanos si existen
        if let Some(sibling) = &self.sibling {
            sibling.post_order_traversal(visit);
        }
    }
    pub fn post_order_traversal_mut(&mut self, visit: &mut dyn FnMut(&mut Node)) {
        // Primero recorremos los hijos dependiendo del tipo de nodo
        match &mut self.node {
            // Si el nodo es una sentencia (Stmt), verificamos su tipo
            Node::Stmt { kind, .. } => match kind {
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.post_order_traversal_mut(visit);
                    // Si existe el bloque "then", lo recorremos
                    if let Some(then) = then_branch {
                        then.post_order_traversal_mut(visit);
                    }
                    // Si existe el bloque "else", lo recorremos
                    if let Some(else_) = else_branch {
                        else_.post_order_traversal_mut(visit);
                    }
                }
                StmtKind::While { condition, body } => {
                    // Visitamos la condición (que es un TreeNode) de manera recursiva
                    condition.post_order_traversal_mut(visit);
                    // Si existe el cuerpo, lo recorremos
                    if let Some(body) = body {
                        body.post_order_traversal_mut(visit);
                    }
                }
                StmtKind::Do { body, condition } => {
                    // Visitamos el cuerpo si existe
                    if let Some(body) = body {
                        body.post_order_traversal_mut(visit);
                    }
                    // Luego visitamos la condición (que es un Node) directamente
                    condition.post_order_traversal_mut(visit);
                }
                StmtKind::Assign { value, .. } => {
                    // Visitamos el valor de la asignación (que es un TreeNode) de manera recursiva
                    value.post_order_traversal_mut(visit);
                }
                StmtKind::In { .. } => {
                    // No tiene hijos
                }
                StmtKind::Out { expression } => {
                    // Para el caso de "Out", visitamos la expresión
                    expression.post_order_traversal_mut(visit);
                }
            },
            // Si el nodo es una expresión (Exp), verificamos su tipo
            Node::Exp { kind, .. } => match kind {
                ExpKind::Op { left, right, .. } => {
                    // Visitamos el lado izquierdo de manera recursiva
                    left.post_order_traversal_mut(visit);
                    // Si hay un lado derecho, también lo visitamos de manera recursiva
                    if let Some(right) = right {
                        right.post_order_traversal_mut(visit);
                    }
                }
                ExpKind::Const { .. } | ExpKind::ConstF { .. } | ExpKind::Id { .. } => {
                    // Estos nodos no tienen hijos
                }
            },
            // Si el nodo es una declaración (Decl), verificamos el tipo
            Node::Decl { kind, .. } => match kind {
                DeclKind::Var { .. } => {
                    // No hay hijos en este caso
                }
            },
        }

        // Finalmente, visitamos el nodo actual
        visit(&mut self.node);

        // Luego, recorremos los hermanos si existen
        if let Some(sibling) = &mut self.sibling {
            sibling.post_order_traversal_mut(visit);
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

impl Node {}

impl Add for NodeValue {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => Some(NodeValue::Int(left + right)),
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left as f32 + right))
            }

            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left + right as f32))
            }

            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left - right))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl Sub for NodeValue {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => Some(NodeValue::Int(left - right)),
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left as f32 - right))
            }

            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left - right as f32))
            }

            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left - right))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl Mul for NodeValue {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => Some(NodeValue::Int(left * right)),
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left as f32 * right))
            }

            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left * right as f32))
            }

            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left * right))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl Rem for NodeValue {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => Some(NodeValue::Int(left % right)),
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left as f32 % right))
            }
            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left % right as f32))
            }
            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left % right))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl Div for NodeValue {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => Some(NodeValue::Int(left / right)),
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left as f32 / right))
            }
            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left / right as f32))
            }
            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left / right))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl PartialEq for NodeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeValue::Int(i), NodeValue::Int(r)) => i == r,
            (NodeValue::Int(i), NodeValue::Float(r)) => *i as f32 == *r,
            (NodeValue::Int(i), NodeValue::Boolean(r)) => (*i == 1) == *r,
            (NodeValue::Float(i), NodeValue::Int(r)) => *i == *r as f32,
            (NodeValue::Float(i), NodeValue::Float(r)) => i == r,
            (NodeValue::Float(i), NodeValue::Boolean(r)) => (*i == 1.0) == *r,
            (NodeValue::Boolean(i), NodeValue::Int(r)) => *i == (*r == 1),
            (NodeValue::Boolean(i), NodeValue::Float(r)) => *i == (*r == 1.0),
            (NodeValue::Boolean(i), NodeValue::Boolean(r)) => i == r,
        }
    }
}

impl PartialOrd for NodeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (NodeValue::Int(l), NodeValue::Int(r)) => l.partial_cmp(r),
            (NodeValue::Int(l), NodeValue::Float(r)) => (*l as f32).partial_cmp(r),
            (NodeValue::Float(l), NodeValue::Int(r)) => l.partial_cmp(&(*r as f32)),
            (NodeValue::Float(l), NodeValue::Float(r)) => l.partial_cmp(r),
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }
}

impl NodeValue {
    pub fn pow(self, other: Self) -> Option<Self> {
        match (self, other) {
            (NodeValue::Int(left), NodeValue::Int(right)) => {
                Some(NodeValue::Int(left.pow(right as u32)))
            }
            (NodeValue::Int(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float((left as f32).powf(right)))
            }
            (NodeValue::Float(left), NodeValue::Int(right)) => {
                Some(NodeValue::Float(left.powi(right)))
            }
            (NodeValue::Float(left), NodeValue::Float(right)) => {
                Some(NodeValue::Float(left.powf(right)))
            }
            (NodeValue::Int(_), NodeValue::Boolean(_))
            | (NodeValue::Float(_), NodeValue::Boolean(_))
            | (NodeValue::Boolean(_), NodeValue::Int(_))
            | (NodeValue::Boolean(_), NodeValue::Float(_))
            | (NodeValue::Boolean(_), NodeValue::Boolean(_)) => None,
        }
    }

    pub fn to_float(self) -> Option<Self> {
        match self {
            NodeValue::Int(value) => Some(NodeValue::Float(value as f32)),
            NodeValue::Float(_) => Some(self),
            NodeValue::Boolean(_) => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Int(value) => value.to_string(),
            NodeValue::Float(value) => value.to_string(),
            NodeValue::Boolean(value) => match value {
                true => 1.to_string(),
                false => 0.to_string(),
            },
        }
    }
}

fn print_tree(node: &TreeNode, indent: usize) {
    let indentation = " ".repeat(indent);
    match &node.node {
        Node::Decl { kind, .. } => match kind {
            DeclKind::Var { typ, name } => {
                println!("{}Decl: Var (Type: {:?}, Name: {})", indentation, typ, name);
            }
        },
        Node::Stmt { kind, .. } => match kind {
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                println!("{}Stmt: If", indentation);
                println!("{}  Condition:", indentation);
                print_tree(condition, indent + 4);
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
                print_tree(condition, indent + 4);
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
                print_tree(condition, indent + 4);
            }
            StmtKind::Assign {
                name,
                value,
                exp_value,
                ..
            } => {
                println!(
                    "{}Stmt: Assign (Name: {}) | Value: {:?}",
                    indentation, name, exp_value
                );
                println!("{}  Value:", indentation);
                print_tree(value, indent + 4);
            }
            StmtKind::In { name } => {
                println!("{}Stmt: In (Name: {})", indentation, name);
            }
            StmtKind::Out { expression } => {
                println!("{}Stmt: Out", indentation);
                println!("{}  Expression:", indentation);
                print_tree(expression, indent + 4);
            }
        },
        Node::Exp { kind, val, typ, .. } => match kind {
            ExpKind::Op { op, left, right } => {
                print!("{}Exp: Op ({:?}) | Type {:?}", indentation, op, typ);
                if let Some(value) = val {
                    match value {
                        NodeValue::Int(i) => println!(" Val: {}", i),
                        NodeValue::Float(f) => println!(" Val: {}", f),
                        NodeValue::Boolean(b) => println!(" Val: {}", b),
                    }
                } else {
                    println!();
                }
                println!("{}  Left:", indentation);
                print_tree(left, indent + 4);
                println!("{}  Right:", indentation);
                if let Some(right_node) = right {
                    print_tree(&right_node, indent + 4);
                }
            }
            ExpKind::Const { value } => {
                println!(
                    "{}Exp: Const (Value: {} | Type: {:?})",
                    indentation, value, typ
                );
            }
            ExpKind::ConstF { value } => {
                println!(
                    "{}Exp: Const Float (Value: {}) | Type: {:?})",
                    indentation, value, typ
                );
            }
            ExpKind::Id { name } => {
                println!("{}Exp: Id (Name: {}) | Type: {:?}", indentation, name, typ);
            }
        },
    }
    if let Some(sibling) = &node.sibling {
        print_tree(sibling, indent);
    }
}

pub fn print_sym_table(table: &HashMap<String, SymbolData>) {
    println!("------ TABLA DE SIMBOLOS --------");
    for (k, v) in table.iter() {
        print!(
            "Variable:  {}  | Position ({},{}) | Type: {:?} | Value: {:?} | Location {} | Usages: ",
            k, v.declaration.lin, v.declaration.col, v.typ, v.value, v.mem_location
        );
        for usage in v.usages.iter() {
            print!("({}, {}),", usage.cursor.lin, usage.cursor.col,)
        }
        println!();
    }
    println!("------ TABLA DE SIMBOLOS --------");
}
