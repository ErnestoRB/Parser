use uuid::Uuid;

use crate::structures::TreeNode;

pub struct CodeGen {
    label_count: u32,
}

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen { label_count: 0 }
    }

    pub fn generate(mut self, node: &TreeNode) -> String {
        return self.gen_node_code(node);
    }

    fn gen_label(&mut self) -> String {
        self.label_count += 1;
        format!("label{}", self.label_count)
    }

    fn gen_node_code(&mut self, node: &TreeNode) -> String {
        let mut code = String::new();
        match &node.node {
            crate::structures::Node::Stmt {
                kind,
                id: _,
                cursor: _,
            } => match &kind {
                crate::structures::StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let else_label = self.gen_label();
                    code.push_str(&format!(
                        "{}\nJMPEQ {}\n{}\n{}:\n{}", // JMPEQ porque se busca que el resultado de la expresión sea 0
                        self.gen_node_code(condition.as_ref()), // primero evaluar condicion
                        else_label, // saltar a rama else si no se cumple la condicion
                        then_branch
                            .as_ref()
                            .map(|n| self.gen_node_code(n.as_ref())) // ejecutar el cuerpo if
                            .unwrap_or_else(|| "".to_string()),
                        else_label, // etiqueta de fin
                        else_branch
                            .as_ref()
                            .map(|n| self.gen_node_code(n.as_ref())) // ejecutar el else
                            .unwrap_or_else(|| "".to_string())
                    ));
                }
                crate::structures::StmtKind::While { condition, body } => {
                    let condition_label = self.gen_label();
                    let end_label = self.gen_label();
                    code.push_str(&format!(
                        "{}:\n{}\nJMPEQ {}\n{}\nJMP {}\n{}:", // JMPEQ porque se busca que el resultado de la expresión sea 0
                        condition_label,                      // primero la etiqueta de condición
                        self.gen_node_code(condition.as_ref()), // evaluar condicion
                        end_label, // saltar al final si no se cumple la condicion
                        body.as_ref()
                            .map(|n| self.gen_node_code(n.as_ref())) // ejecutar el cuerpo
                            .unwrap_or_else(|| "".to_string()),
                        condition_label, // volver a evaluar la condición
                        end_label        // etiqueta de fin
                    ));
                }
                crate::structures::StmtKind::Do { body, condition } => {
                    let body_label = self.gen_label();
                    code.push_str(&format!(
                        "{}:\n{}\n{}\nJMPEQ {}",
                        body_label,
                        body.as_ref()
                            .map(|n| self.gen_node_code(n.as_ref()))
                            .unwrap_or_else(|| "".to_string()),
                        self.gen_node_code(condition.as_ref()),
                        body_label
                    ));
                }
                crate::structures::StmtKind::Assign {
                    name,
                    exp_value: _,
                    value,
                } => code.push_str(&format!(
                    "{}\nSTORE_VAR {}",
                    self.gen_node_code(value.as_ref()),
                    name
                )),
                crate::structures::StmtKind::In { name } => {
                    code.push_str(&format!("READ\nSTORE_VAR {}", name))
                }
                crate::structures::StmtKind::Out { expression } => {
                    let expression_code = self.gen_node_code(expression.as_ref());
                    code.push_str(&format!("{}\nPRINT", expression_code))
                }
            },
            crate::structures::Node::Exp {
                kind,
                typ: _,
                id: _,
                cursor: _,
                val,
            } => match &kind {
                crate::structures::ExpKind::Op { op, left, right } => match val {
                    Some(v) => {
                        code.push_str(&format!("LOAD_CONST {}", v.to_string()));
                    }
                    None => {
                        let left_code = self.gen_node_code(left.as_ref());

                        let right_code = right.as_ref().map(|n| self.gen_node_code(n.as_ref()));

                        let op_ins = match op {
                            scanner::data::TokenType::SUM => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "ADD")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::MIN => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "SUB")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::TIMES => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "MUL")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::DIV => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "DIV")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::MODULUS => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "MOD")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::POWER => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "POW")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::LT
                            | scanner::data::TokenType::LE
                            | scanner::data::TokenType::GT
                            | scanner::data::TokenType::GE
                            | scanner::data::TokenType::NE
                            | scanner::data::TokenType::EQ => {
                                let jmp_ins = match op {
                                    scanner::data::TokenType::LT => "JMPLT",
                                    scanner::data::TokenType::LE => "JMPLE",
                                    scanner::data::TokenType::GT => "JMPGT",
                                    scanner::data::TokenType::GE => "JMPGE",
                                    scanner::data::TokenType::NE => "JMPNE",
                                    scanner::data::TokenType::EQ => "JMPEQ",
                                    _ => panic!(
                                        "Operación no válida. Bug en el analizador sintáctico"
                                    ),
                                };
                                let true_label = self.gen_label();
                                let end_label = self.gen_label();
                                if let Some(r_code) = right_code {
                                    &format!(
                                        "{}\n{}\n{}\n{} {}\n{}\nJMP {}\n{}:\n{}\n{}:",
                                        left_code,
                                        r_code,
                                        "SUB",
                                        jmp_ins,
                                        true_label,
                                        "LOAD_CONST 0",
                                        end_label,
                                        true_label,
                                        "LOAD_CONST 1",
                                        end_label
                                    )
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::AND => {
                                if let Some(r_code) = right_code {
                                    &format!("{}\n{}\n{}", left_code, r_code, "MUL")
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::OR => {
                                if let Some(r_code) = right_code {
                                    &format!(
                                        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                                        &left_code,
                                        &r_code,
                                        "MUL",
                                        &left_code,
                                        &r_code,
                                        "SUM",
                                        "SUB"
                                    )
                                } else {
                                    ""
                                }
                            }
                            scanner::data::TokenType::NEG => {
                                &format!("{}\n{}\n{}", "LOAD_CONST 1", &left_code, "SUB")
                            }
                            _ => panic!("Operación no válida. Bug en el analizador sintáctico"),
                        };
                        code.push_str(op_ins);
                    }
                },
                crate::structures::ExpKind::Const { value } => {
                    code.push_str(&format!("LOAD_CONST {}", value))
                }
                crate::structures::ExpKind::ConstF { value } => {
                    code.push_str(&format!("LOAD_CONST {}", value))
                }

                crate::structures::ExpKind::Id { name } => {
                    code.push_str(&format!("LOAD_VAR {}", name))
                }
            },
            crate::structures::Node::Decl {
                kind,
                id: _,
                cursor: _,
            } => match kind {
                crate::structures::DeclKind::Var { typ: _, name } => {
                    code.push_str(&format!("LOAD_CONST 1\nSTORE_VAR {}", name))
                }
            },
        };
        code.push_str("\n");
        if let Some(next) = node.sibling.as_ref() {
            code.push_str(self.gen_node_code(next).as_str());
        }
        code
    }
}
