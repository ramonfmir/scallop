use scallop_core::common::aggregate_op::AggregateOp;
use scallop_core::runtime::dynamic::dataflow::*;
use scallop_core::runtime::dynamic::*;
use scallop_core::runtime::env::*;
use scallop_core::runtime::provenance::*;
use scallop_core::testing::*;

#[test]
fn test_dynamic_aggregate_count_1() {
  let ctx = unit::UnitProvenance::default();
  let rt = RuntimeEnvironment::default();

  // Relations
  let mut source_1 = DynamicRelation::<unit::UnitProvenance>::new();
  let mut source_2 = DynamicRelation::<unit::UnitProvenance>::new();
  let mut target = DynamicRelation::<unit::UnitProvenance>::new();

  // Initial
  source_1.insert_untagged(&ctx, vec![(0i8, 1i8), (1i8, 2i8), (3i8, 4i8), (3i8, 5i8)]);
  source_2.insert_untagged(&ctx, vec![(1i8, 1i8), (1i8, 2i8), (3i8, 5i8)]);

  // Iterate until fixpoint
  while source_1.changed(&ctx) || source_2.changed(&ctx) || target.changed(&ctx) {
    target.insert_dataflow_recent(
      &ctx,
      &DynamicDataflow::from(&source_1).intersect(DynamicDataflow::from(&source_2), &ctx),
      &rt,
    )
  }

  let completed_target = target.complete(&ctx);

  let mut first_time = true;
  let mut agg = DynamicRelation::<unit::UnitProvenance>::new();
  while agg.changed(&ctx) || first_time {
    agg.insert_dataflow_recent(
      &ctx,
      &DynamicAggregationDataflow::single(
        AggregateOp::Count.into(),
        DynamicDataflow::dynamic_collection(&completed_target, first_time),
        &ctx,
      )
      .into(),
      &rt,
    );
    first_time = false;
  }

  expect_collection(&agg.complete(&ctx), vec![2usize]);
}
