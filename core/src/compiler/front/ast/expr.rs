use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
  Constant(Constant),
  Variable(Variable),
  Wildcard(Wildcard),
  Binary(BinaryExpr),
  Unary(UnaryExpr),
  IfThenElse(IfThenElseExpr),
  Call(CallExpr),
}

impl Expr {
  /// Create a unary expression
  pub fn unary(op: UnaryOp, expr: Expr) -> Self {
    Self::Unary(UnaryExpr::default(UnaryExprNode {
      op,
      op1: Box::new(expr),
    }))
  }

  /// Create a binary expression
  pub fn binary(op: BinaryOp, op1: Expr, op2: Expr) -> Self {
    Self::Binary(BinaryExpr::default(BinaryExprNode {
      op,
      op1: Box::new(op1),
      op2: Box::new(op2),
    }))
  }

  /// Create a constant boolean expression
  pub fn boolean(b: bool) -> Self {
    Self::Constant(ConstantNode::Boolean(b).into())
  }

  /// Create an expression which is a constant of boolean true value
  pub fn boolean_true() -> Self {
    Self::Constant(ConstantNode::Boolean(true).into())
  }

  /// Create an expression which is a constant of boolean false value
  pub fn boolean_false() -> Self {
    Self::Constant(ConstantNode::Boolean(false).into())
  }

  pub fn location(&self) -> &AstNodeLocation {
    match self {
      Self::Constant(c) => c.location(),
      Self::Variable(v) => v.location(),
      Self::Wildcard(w) => w.location(),
      Self::Binary(b) => b.location(),
      Self::Unary(u) => u.location(),
      Self::IfThenElse(i) => i.location(),
      Self::Call(c) => c.location(),
    }
  }

  pub fn is_constant(&self) -> bool {
    match self {
      Self::Constant(_) => true,
      _ => false,
    }
  }

  pub fn is_variable(&self) -> bool {
    match self {
      Self::Variable(_) => true,
      _ => false,
    }
  }

  pub fn is_wildcard(&self) -> bool {
    match self {
      Self::Wildcard(_) => true,
      _ => false,
    }
  }

  pub fn is_complex_expr(&self) -> bool {
    match self {
      Self::Binary(_) | Self::Unary(_) => true,
      _ => false,
    }
  }

  pub fn collect_used_variables(&self) -> Vec<Variable> {
    let mut vars = vec![];
    self.collect_used_variables_helper(&mut vars);
    vars
  }

  fn collect_used_variables_helper(&self, vars: &mut Vec<Variable>) {
    match self {
      Self::Binary(b) => {
        b.op1().collect_used_variables_helper(vars);
        b.op2().collect_used_variables_helper(vars);
      }
      Self::Unary(u) => {
        u.op1().collect_used_variables_helper(vars);
      }
      Self::Call(c) => {
        for a in c.iter_args() {
          a.collect_used_variables_helper(vars);
        }
      }
      Self::Constant(_) => {}
      Self::Wildcard(_) => {}
      Self::IfThenElse(i) => {
        i.cond().collect_used_variables_helper(vars);
        i.then_br().collect_used_variables_helper(vars);
        i.else_br().collect_used_variables_helper(vars);
      }
      Self::Variable(v) => {
        vars.push(v.clone());
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct VariableNode {
  pub name: Identifier,
}

impl VariableNode {
  pub fn new(name: Identifier) -> Self {
    Self { name }
  }
}

pub type Variable = AstNode<VariableNode>;

impl Variable {
  pub fn default_with_name(name: String) -> Self {
    Self::default(VariableNode::new(Identifier::default(IdentifierNode::new(name))))
  }

  pub fn name(&self) -> &str {
    self.node.name.name()
  }
}

impl std::fmt::Display for Variable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.name())
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct VariableBindingNode {
  pub name: Identifier,
  pub ty: Option<Type>,
}

pub type VariableBinding = AstNode<VariableBindingNode>;

impl VariableBinding {
  pub fn name(&self) -> &str {
    self.node.name.name()
  }

  pub fn to_variable(&self) -> Variable {
    Variable {
      loc: self.loc.clone(),
      node: VariableNode::new(self.node.name.clone()),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct WildcardNode;

pub type Wildcard = AstNode<WildcardNode>;

#[doc(hidden)]
pub type BinaryOpNode = crate::common::binary_op::BinaryOp;

pub type BinaryOp = AstNode<BinaryOpNode>;

impl BinaryOp {
  pub fn default_eq() -> Self {
    Self::default(crate::common::binary_op::BinaryOp::Eq)
  }

  pub fn is_arith(&self) -> bool {
    self.node.is_arith()
  }

  pub fn is_add_sub(&self) -> bool {
    self.node.is_add_sub()
  }

  pub fn is_logical(&self) -> bool {
    self.node.is_logical()
  }

  pub fn is_eq_neq(&self) -> bool {
    self.node.is_eq_neq()
  }

  pub fn is_eq(&self) -> bool {
    self.node.is_eq()
  }

  pub fn is_numeric_cmp(&self) -> bool {
    self.node.is_numeric_cmp()
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct BinaryExprNode {
  pub op: BinaryOp,
  pub op1: Box<Expr>,
  pub op2: Box<Expr>,
}

pub type BinaryExpr = AstNode<BinaryExprNode>;

impl BinaryExpr {
  pub fn op(&self) -> &BinaryOp {
    &self.node.op
  }

  pub fn op1(&self) -> &Expr {
    &self.node.op1
  }

  pub fn op2(&self) -> &Expr {
    &self.node.op2
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum UnaryOpNode {
  Neg,
  Pos,
  Not,
  TypeCast(Type),
}

pub type UnaryOp = AstNode<UnaryOpNode>;

impl UnaryOp {
  pub fn default_not() -> Self {
    Self::default(UnaryOpNode::Not)
  }

  pub fn is_pos_neg(&self) -> bool {
    match &self.node {
      UnaryOpNode::Pos | UnaryOpNode::Neg => true,
      _ => false,
    }
  }

  pub fn is_not(&self) -> bool {
    match &self.node {
      UnaryOpNode::Not => true,
      _ => false,
    }
  }

  pub fn cast_to_type(&self) -> Option<&Type> {
    match &self.node {
      UnaryOpNode::TypeCast(t) => Some(t),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct UnaryExprNode {
  pub op: UnaryOp,
  pub op1: Box<Expr>,
}

pub type UnaryExpr = AstNode<UnaryExprNode>;

impl UnaryExpr {
  pub fn op(&self) -> &UnaryOp {
    &self.node.op
  }

  pub fn op1(&self) -> &Expr {
    &self.node.op1
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct IfThenElseExprNode {
  pub cond: Box<Expr>,
  pub then_br: Box<Expr>,
  pub else_br: Box<Expr>,
}

pub type IfThenElseExpr = AstNode<IfThenElseExprNode>;

impl IfThenElseExpr {
  pub fn cond(&self) -> &Expr {
    &self.node.cond
  }

  pub fn then_br(&self) -> &Expr {
    &self.node.then_br
  }

  pub fn else_br(&self) -> &Expr {
    &self.node.else_br
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct CallExprNode {
  pub function_identifier: FunctionIdentifier,
  pub args: Vec<Expr>,
}

impl CallExprNode {
  pub fn new(function_identifier: FunctionIdentifier, args: Vec<Expr>) -> Self {
    Self {
      function_identifier,
      args,
    }
  }
}

pub type CallExpr = AstNode<CallExprNode>;

impl CallExpr {
  pub fn num_args(&self) -> usize {
    self.node.args.len()
  }

  pub fn iter_args(&self) -> impl Iterator<Item = &Expr> {
    self.node.args.iter()
  }

  pub fn iter_args_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
    self.node.args.iter_mut()
  }

  pub fn function_identifier(&self) -> &FunctionIdentifier {
    &self.node.function_identifier
  }

  pub fn function_identifier_mut(&mut self) -> &mut FunctionIdentifier {
    &mut self.node.function_identifier
  }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub struct FunctionIdentifierNode {
  pub id: Identifier,
}

/// The identifier of a function, i.e. `$abs`
pub type FunctionIdentifier = AstNode<FunctionIdentifierNode>;

impl FunctionIdentifier {
  pub fn name(&self) -> &str {
    self.node.id.name()
  }
}
