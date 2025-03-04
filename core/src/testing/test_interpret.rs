use crate::common::tuple::Tuple;
use crate::integrate::*;
use crate::runtime::database::*;
use crate::runtime::monitor;
use crate::runtime::provenance::*;
use crate::utils::*;

use super::*;

pub fn expect_interpret_result<T: Into<Tuple> + Clone>(s: &str, (p, e): (&str, Vec<T>)) {
  let actual = interpret_string(s.to_string()).expect("Compile Error");
  expect_output_collection(p, actual.get_output_collection_ref(p).unwrap(), e);
}

pub fn expect_interpret_result_with_setup<T, F>(s: &str, f: F, (p, e): (&str, Vec<T>))
where
  T: Into<Tuple> + Clone,
  F: FnOnce(&mut extensional::ExtensionalDatabase<unit::UnitProvenance>),
{
  let prov = unit::UnitProvenance::default();
  let mut interpret_ctx = InterpretContext::<_, RcFamily>::new(s.to_string(), prov).expect("Compilation error");
  f(interpret_ctx.edb());
  interpret_ctx.run().expect("Runtime error");
  let idb = interpret_ctx.idb();
  expect_output_collection(p, idb.get_output_collection_ref(p).unwrap(), e);
}

pub fn expect_interpret_result_with_tag<Prov, T, F>(s: &str, ctx: Prov, (p, e): (&str, Vec<(Prov::OutputTag, T)>), f: F)
where
  Prov: Provenance,
  T: Into<Tuple> + Clone,
  F: Fn(&Prov::OutputTag, &Prov::OutputTag) -> bool,
{
  let actual = interpret_string_with_ctx(s.to_string(), ctx).expect("Interpret Error");
  expect_output_collection_with_tag(p, actual.get_output_collection_ref(p).unwrap(), e, f);
}

/// Expect the given program to produce an empty relation `p`
///
/// ``` rust
/// # use scallop_core::testing::*;
/// expect_interpret_empty_result("type edge(i32, i32)", "edge")
/// ```
pub fn expect_interpret_empty_result(s: &str, p: &str) {
  let actual = interpret_string(s.to_string()).expect("Compile Error");
  assert!(
    actual.get_output_collection_ref(p).unwrap().is_empty(),
    "The relation `{}` is not empty",
    p
  )
}

/// Expect the given program to produce the expected relation/collections.
/// Panics if the program fails to compile/execute, or it does not produce the expected results.
pub fn expect_interpret_multi_result(s: &str, expected: Vec<(&str, TestCollection)>) {
  let actual = interpret_string(s.to_string()).expect("Compile Error");
  for (p, a) in expected {
    expect_output_collection(p, actual.get_output_collection_ref(p).unwrap(), a);
  }
}

/// Expect the given program to be executed within a given iteration limit.
/// It panics if the program uses an iteration count more than the limit.
pub fn expect_interpret_within_iter_limit(s: &str, iter_limit: usize) {
  let prov = unit::UnitProvenance::default();
  let monitor = monitor::IterationCheckingMonitor::new(iter_limit);
  interpret_string_with_ctx_and_monitor(s.to_string(), prov, &monitor).expect("Interpret Error");
}
