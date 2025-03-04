use itertools::Itertools;
use std::collections::*;

use super::*;
use crate::compiler::front::ast::*;
use crate::compiler::front::visitor::*;

#[derive(Debug, Clone)]
pub struct RuleContext {
  pub head_vars: Vec<(String, Loc)>,
  pub body: DisjunctionContext,
}

impl RuleContext {
  pub fn from_rule(rule: &Rule) -> Self {
    let head_vars = collect_vars_in_head(rule.head());
    let body = DisjunctionContext::from_formula(rule.body());
    Self { head_vars, body }
  }

  pub fn from_qualified(bindings: &Vec<VariableBinding>, args: &Vec<Variable>, body: &Formula) -> Self {
    let bindings = bindings
      .iter()
      .map(|b| (b.name().to_string(), b.location().clone()))
      .collect::<Vec<_>>();
    let args = args
      .iter()
      .map(|v| (v.name().to_string(), v.location().clone()))
      .collect();
    let head_vars = vec![bindings, args].concat();
    let body = DisjunctionContext::from_formula(body);
    Self { head_vars, body }
  }

  pub fn compute_boundness(
    &self,
    predicate_bindings: &ForeignPredicateBindings,
    bounded_exprs: &Vec<Expr>,
  ) -> Result<BTreeSet<String>, Vec<BoundnessAnalysisError>> {
    let bounded_vars = self.body.compute_boundness(predicate_bindings, bounded_exprs)?;
    for (var_name, var_loc) in &self.head_vars {
      if !bounded_vars.contains(var_name) {
        let err = BoundnessAnalysisError::HeadExprUnbound { loc: var_loc.clone() };
        return Err(vec![err]);
      }
    }
    Ok(bounded_vars)
  }
}

#[derive(Debug, Clone)]
pub struct DisjunctionContext {
  pub conjuncts: Vec<ConjunctionContext>,
}

impl DisjunctionContext {
  pub fn from_formula(formula: &Formula) -> Self {
    let conjuncts: Vec<ConjunctionContext> = match formula {
      Formula::Disjunction(d) => d.args().map(|a| Self::from_formula(a).conjuncts).flatten().collect(),
      Formula::Conjunction(c) => {
        let ctxs = c.args().map(|a| Self::from_formula(a).conjuncts);
        let cp = ctxs.multi_cartesian_product();
        cp.map(ConjunctionContext::join).collect()
      }
      Formula::Implies(_) => {
        panic!("Unexpected `implies` visited during boundness analysis; implies should be rewritten by previous transformations")
      }
      Formula::Atom(a) => vec![ConjunctionContext::from_atom(a)],
      Formula::NegAtom(a) => vec![ConjunctionContext::from_neg_atom(a)],
      Formula::Constraint(a) => vec![ConjunctionContext::from_constraint(a)],
      Formula::Reduce(r) => vec![ConjunctionContext::from_reduce(r)],
      Formula::ForallExistsReduce(_) => {
        panic!("Unexpected `forall/exists` visited during boundness analysis; forall/exists should be rewritten by previous transformations")
      }
    };
    Self { conjuncts }
  }

  pub fn compute_boundness(
    &self,
    predicate_bindings: &ForeignPredicateBindings,
    bounded_exprs: &Vec<Expr>,
  ) -> Result<BTreeSet<String>, Vec<BoundnessAnalysisError>> {
    if self.conjuncts.is_empty() {
      Ok(BTreeSet::new())
    } else if self.conjuncts.len() == 1 {
      self.conjuncts[0].compute_boundness(predicate_bindings, bounded_exprs)
    } else {
      let set1 = self.conjuncts[0].compute_boundness(predicate_bindings, bounded_exprs)?;
      let other_sets = self.conjuncts[1..]
        .iter()
        .map(|c| c.compute_boundness(predicate_bindings, bounded_exprs))
        .collect::<Result<Vec<BTreeSet<_>>, _>>()?;
      Ok(
        set1
          .into_iter()
          .filter(|v| other_sets.iter().all(|s| s.contains(v)))
          .collect(),
      )
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct ConjunctionContext {
  pub pos_atoms: Vec<Formula>,
  pub neg_atoms: Vec<Formula>,
  pub agg_contexts: Vec<AggregationContext>,
}

impl ConjunctionContext {
  pub fn join(conjs: Vec<Self>) -> Self {
    conjs.into_iter().fold(Self::default(), |acc, new| Self {
      pos_atoms: vec![acc.pos_atoms, new.pos_atoms].concat(),
      neg_atoms: vec![acc.neg_atoms, new.neg_atoms].concat(),
      agg_contexts: vec![acc.agg_contexts, new.agg_contexts].concat(),
    })
  }

  pub fn from_atom(atom: &Atom) -> Self {
    Self {
      pos_atoms: vec![Formula::Atom(atom.clone())],
      neg_atoms: vec![],
      agg_contexts: vec![],
    }
  }

  pub fn from_neg_atom(neg_atom: &NegAtom) -> Self {
    Self {
      pos_atoms: vec![],
      neg_atoms: vec![Formula::NegAtom(neg_atom.clone())],
      agg_contexts: vec![],
    }
  }

  pub fn from_constraint(constraint: &Constraint) -> Self {
    Self {
      pos_atoms: vec![Formula::Constraint(constraint.clone())],
      neg_atoms: vec![],
      agg_contexts: vec![],
    }
  }

  pub fn from_reduce(reduce: &Reduce) -> Self {
    Self {
      pos_atoms: vec![],
      neg_atoms: vec![],
      agg_contexts: vec![AggregationContext::from_reduce(reduce)],
    }
  }

  pub fn compute_boundness(
    &self,
    predicate_bindings: &ForeignPredicateBindings,
    bounded_exprs: &Vec<Expr>,
  ) -> Result<BTreeSet<String>, Vec<BoundnessAnalysisError>> {
    let mut local_ctx = LocalBoundnessAnalysisContext::new(predicate_bindings);

    // First check if the aggregation's boundness is okay
    for agg_context in &self.agg_contexts {
      // The bounded variables inside the aggregation is part of the bounded vars
      let bounded_args = agg_context.compute_boundness(predicate_bindings, bounded_exprs)?;
      local_ctx.bounded_variables.extend(bounded_args);
    }

    // Then check the positive
    for formula in &self.pos_atoms {
      local_ctx.walk_formula(formula);
    }

    // Walk the bounded expressions
    for expr in bounded_exprs {
      local_ctx.walk_expr(expr);
      local_ctx.expr_boundness.insert(expr.location().clone(), true);
    }

    // Compute boundness
    local_ctx.compute_boundness();
    if local_ctx.errors.is_empty() {
      Ok(local_ctx.bounded_variables)
    } else {
      Err(local_ctx.errors)
    }
  }
}

#[derive(Debug, Clone)]
pub struct AggregationContext {
  pub result_vars: Vec<Variable>,
  pub binding_vars: Vec<String>,
  pub arg_vars: Vec<Variable>,
  pub body: Box<RuleContext>,
  pub body_formula: Formula,
  pub joined_body: Box<RuleContext>,
  pub joined_body_formula: Formula,
  pub group_by: Option<(Box<RuleContext>, Vec<Variable>, Formula)>,
  pub aggregate_op: ReduceOperatorNode,
}

impl AggregationContext {
  pub fn left_variable_names(&self) -> BTreeSet<String> {
    self.result_vars.iter().map(|v| v.name().to_string()).collect()
  }

  pub fn binding_variable_names(&self) -> BTreeSet<String> {
    self.binding_vars.iter().cloned().collect()
  }

  pub fn argument_variable_names(&self) -> BTreeSet<String> {
    self.arg_vars.iter().map(|n| n.name().to_string()).collect()
  }

  pub fn group_by_head_variable_names(&self) -> BTreeSet<String> {
    if let Some((_, vars, _)) = &self.group_by {
      vars.iter().map(|n| n.name().to_string()).collect()
    } else {
      BTreeSet::new()
    }
  }

  pub fn from_reduce(reduce: &Reduce) -> Self {
    // Merge the body and the group_by formula if presented
    let body = RuleContext::from_qualified(reduce.bindings(), reduce.args(), reduce.body());
    let body_formula = reduce.body().clone();

    // Get a joined body formula for both body part and group_by part
    let joined_body_formula = if let Some((_, group_by_formula)) = reduce.group_by() {
      Formula::conjunction(vec![reduce.body().clone(), group_by_formula.clone()])
    } else {
      reduce.body().clone()
    };
    let joined_body = RuleContext::from_qualified(reduce.bindings(), reduce.args(), &joined_body_formula);

    // Get the group_by context
    let group_by = reduce.group_by().map(|(bindings, formula)| {
      let ctx = RuleContext::from_qualified(bindings, &vec![], formula);
      let vars = bindings.iter().map(|b| b.to_variable()).collect::<Vec<_>>();
      (Box::new(ctx), vars, formula.clone())
    });

    // Construct self
    Self {
      result_vars: reduce.left_variables().cloned().collect(),
      binding_vars: reduce.binding_names().map(|n| n.to_string()).collect(),
      arg_vars: reduce.args().clone(),
      body: Box::new(body),
      body_formula,
      joined_body: Box::new(joined_body),
      joined_body_formula,
      group_by,
      aggregate_op: reduce.operator().node.clone(),
    }
  }

  pub fn compute_boundness(
    &self,
    predicate_bindings: &ForeignPredicateBindings,
    bounded_exprs: &Vec<Expr>,
  ) -> Result<HashSet<String>, Vec<BoundnessAnalysisError>> {
    // Construct the bounded
    let mut bounded = HashSet::new();

    // If group_by is presented, check the gruop_by binding variables are properly bounded
    if let Some((group_by_ctx, _, _)) = &self.group_by {
      group_by_ctx.compute_boundness(predicate_bindings, bounded_exprs)?;
    }

    // Add all the bounded variables in the aggregation body
    bounded.extend(self.joined_body.compute_boundness(predicate_bindings, bounded_exprs)?);

    // Remove the qualified variable
    for binding_name in &self.binding_vars {
      bounded.remove(binding_name);
    }

    // Check if the arguments are bounded
    for arg_var in &self.arg_vars {
      if !bounded.contains(arg_var.name()) {
        let err = BoundnessAnalysisError::ReduceArgUnbound {
          loc: arg_var.location().clone(),
        };
        return Err(vec![err]);
      }
    }

    // Add args and result variables
    bounded.extend(self.result_vars.iter().map(|v| v.name().to_string()));
    bounded.extend(self.arg_vars.iter().map(|v| v.name().to_string()));

    Ok(bounded)
  }
}

fn collect_vars_in_head(head: &RuleHead) -> Vec<(String, Loc)> {
  match &head.node {
    RuleHeadNode::Atom(atom) => collect_vars_in_atom(atom),
    RuleHeadNode::Disjunction(d) => d.iter().map(collect_vars_in_atom).flatten().collect(),
  }
}

fn collect_vars_in_atom(atom: &Atom) -> Vec<(String, Loc)> {
  atom.iter_arguments().map(collect_vars_in_expr).flatten().collect()
}

fn collect_vars_in_expr(expr: &Expr) -> Vec<(String, Loc)> {
  match expr {
    Expr::Binary(b) => vec![collect_vars_in_expr(b.op1()), collect_vars_in_expr(b.op2())].concat(),
    Expr::Unary(u) => collect_vars_in_expr(u.op1()),
    Expr::Variable(v) => vec![(v.name().to_string(), v.location().clone())],
    Expr::Constant(_) => vec![],
    Expr::Wildcard(_) => vec![],
    Expr::IfThenElse(i) => vec![
      collect_vars_in_expr(i.cond()),
      collect_vars_in_expr(i.then_br()),
      collect_vars_in_expr(i.else_br()),
    ]
    .concat(),
    Expr::Call(c) => c.iter_args().map(|a| collect_vars_in_expr(a)).concat(),
  }
}
