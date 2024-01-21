#![cfg(test)]
use super::*;
use crate::cache::Cache;

use alloc::boxed::Box;

impl Task<()> for () {
    fn execute(&mut self, _: &BTreeMap<&str, ()>) -> Result<(), ExecuteError> {
        Ok(())
    }
}

struct DepTask(&'static [&'static str]);

impl Task<()> for DepTask {
    fn execute(&mut self, _: &BTreeMap<&str, ()>) -> Result<(), ExecuteError> {
        Ok(())
    }
    fn dependencies(&self) -> &'static [&'static str] {
        self.0
    }
}

#[test]
fn test_trait_semantics() {
    let mut d: Box<dyn Task<()>> = Box::new(());
    let mut map = BTreeMap::new();
    d.execute(&mut map).unwrap();
}

#[test]
fn test_closure_semantics() {
    let mut d = Cache::from_closure(|_| (0..10).sum::<usize>());
    let mut map = BTreeMap::new();

    let result = d.get(&mut map).unwrap();
    assert_eq!(*result, 45);
}

#[test]
fn test_get() {
    let mut dsk = DSK::new();
    let mut cache = BTreeMap::new();

    dsk.add_task("X", |_: &BTreeMap<&str, usize>| (0..10).sum::<usize>())
        .unwrap();
    assert_eq!(dsk.execute("X", &mut cache).unwrap(), 45);
}

#[test]
fn test_cyclic_dep_checker() {
    let mut dsk = DSK::new();

    dsk.add_task("X", DepTask(&["Y"])).unwrap();
    dsk.add_task("Y", DepTask(&["Z"])).unwrap();
    dsk.add_task("Z", DepTask(&["B"])).unwrap();

    assert!(dsk.add_task("B", DepTask(&["X"])).is_err());
}

#[test]
fn converging_dependencies_test() {
    let mut dsk = DSK::new();

    dsk.add_task("Z", DepTask(&[])).unwrap();
    dsk.add_task("U", DepTask(&[])).unwrap();

    dsk.add_task("X", DepTask(&["Y"])).unwrap();
    dsk.add_task("Y", DepTask(&["Z", "U"])).unwrap();
    dsk.add_task("A", DepTask(&["U"])).unwrap();
    assert!(dsk.add_task("B", DepTask(&["Y", "X"])).is_ok());
}

#[test]
fn diverging_dependencies_test() {
    let mut dsk = DSK::new();

    dsk.add_task("A", DepTask(&[])).unwrap();
    dsk.add_task("B", DepTask(&[])).unwrap();

    dsk.add_task("T", DepTask(&["U"])).unwrap();
    dsk.add_task("U", DepTask(&["W", "A"])).unwrap();
    dsk.add_task("W", DepTask(&["X"])).unwrap();
    assert!(dsk.add_task("X", DepTask(&["T"])).is_err());
}

#[test]
fn test_dsk_cull() {
    let mut dsk = DSK::new();

    dsk.add_task("X", DepTask(&["Y"])).unwrap();
    dsk.add_task("Y", DepTask(&["U"])).unwrap();
    dsk.add_task("U", DepTask(&[])).unwrap();
    dsk.add_task("A", DepTask(&["U"])).unwrap();
    dsk.add_task("Z", DepTask(&["A"])).unwrap();

    let culled = dsk.cull(&["X", "U"]).unwrap();

    assert!(culled.0.contains_key("X"));
    assert!(culled.0.contains_key("U"));
    assert!(culled.0.contains_key("Y"));
    assert!(!culled.0.contains_key("A"));
    assert!(!culled.0.contains_key("Z"));
}
