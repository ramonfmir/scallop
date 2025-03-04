use crate::common::tuple::*;
use crate::runtime::env::*;
use crate::runtime::provenance::*;

use super::*;

#[derive(Clone)]
pub struct DynamicUntaggedVec<'a, Prov: Provenance> {
  pub ctx: &'a Prov,
  pub tuples: &'a Vec<Tuple>,
}

impl<'a, Prov: Provenance> DynamicUntaggedVec<'a, Prov> {
  pub fn new(ctx: &'a Prov, tuples: &'a Vec<Tuple>) -> Self {
    Self { ctx, tuples }
  }

  pub fn iter_recent(&self, _: &RuntimeEnvironment) -> DynamicBatches<'a, Prov> {
    DynamicBatches::single(DynamicBatch::untagged_vec(self.ctx, self.tuples.iter()))
  }

  pub fn iter_stable(&self, _: &RuntimeEnvironment) -> DynamicBatches<'a, Prov> {
    DynamicBatches::Empty
  }
}
