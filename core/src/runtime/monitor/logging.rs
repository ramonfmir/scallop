use colored::*;

use crate::runtime::provenance::Provenance;

use super::*;

pub struct LoggingMonitor;

impl LoggingMonitor {
  pub fn info(&self, s: &str) {
    println!("[Info] {}", s.color(Color::Cyan));
  }

  pub fn warning(&self, s: &str) {
    println!("[Warn] {}", s.color(Color::Yellow));
  }

  pub fn error(&self, s: &str) {
    println!("[Error] {}", s.color(Color::Red));
  }
}

impl<Prov: Provenance> Monitor<Prov> for LoggingMonitor {
  fn observe_executing_stratum(&self, stratum_id: usize) {
    self.info(&format!("executing stratum #{}", stratum_id))
  }

  fn observe_stratum_iteration(&self, iteration_count: usize) {
    self.info(&format!("iteration #{}", iteration_count))
  }

  fn observe_loading_relation_from_edb(&self, relation: &str) {
    self.info(&format!("loading relation `{}` from EDB", relation))
  }

  fn observe_loading_relation_from_idb(&self, relation: &str) {
    self.info(&format!("loading relation `{}` from IDB", relation))
  }

  fn observe_recovering_relation(&self, relation: &str) {
    self.info(&format!("recovering relation `{}`", relation))
  }
}
