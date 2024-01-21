#![no_std]
#![deny(missing_docs)]

//! A simple Task Execution and Dependency management crate, reminiscent of dask.py.

mod cache;
mod dsk;
mod error;

mod tests;

pub use cache::Cache;
pub use dsk::*;
pub use error::ExecuteError;

extern crate alloc;
use alloc::collections::BTreeMap;

/// A Task is a unit of work that can be executed.
/// It can have dependencies on other tasks.
pub trait Task<O> {
    /// Execute this task and return the result.
    /// The cache is a reference to a HashMap that contains the results of tasks' whose results are needed by this
    /// task. These tasks are called dependencies. And are listed in the `dependencies` method.
    fn execute(&mut self, cache: &BTreeMap<&str, O>) -> Result<O, error::ExecuteError>;

    /// Lists out this task's dependencies.
    /// These tasks' results are available later in the `cache` argument of the `execute` method.
    fn dependencies(&self) -> &'static [&'static str] {
        &[]
    }
}

/// Any closure is automatically a Task
impl<T, F: Fn(&BTreeMap<&str, T>) -> T> Task<T> for F {
    fn execute(&mut self, cache: &BTreeMap<&str, T>) -> Result<T, error::ExecuteError> {
        Ok(self(cache))
    }
}

/// A NullTask is a task that does nothing.
/// Trying to execute it will result in an error.
/// It is used for structural  purposes.
pub struct NullTask;

impl<T> Task<T> for NullTask {
    fn execute(&mut self, _: &BTreeMap<&str, T>) -> Result<T, error::ExecuteError> {
        Err(ExecuteError::NullTask)
    }
}
