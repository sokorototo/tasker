use std::collections::BTreeMap;
use std::fmt;

use super::error;
use super::Task;
use crate::NullTask;

/// A Cache is a task that once executed, caches it's result
pub struct Cache<'task, V> {
	result: Option<V>,
	calculation: Box<dyn Task<V> + 'task>,
}

impl<'task, V: fmt::Debug> fmt::Debug for Cache<'task, V> {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		f.debug_struct("Cache")
			.field("result", &self.result)
			.field("calculation", &format!("Box<dyn Task<{}>>", std::any::type_name::<V>()))
			.finish()
	}
}

impl<'task, V> Cache<'task, V> {
	/// Create a new Cache from the given task.
	pub fn from(task: impl Task<V> + 'task) -> Self {
		Self {
			result: None,
			calculation: Box::new(task),
		}
	}

	/// Create a new Cache with a pre-defined result.
	pub fn from_result(result: V) -> Cache<'task, V> {
		Cache {
			result: Some(result),
			calculation: Box::new(NullTask),
		}
	}

	/// Create a new Cache using a closure.
	pub fn from_closure<F: Fn(&BTreeMap<&str, V>) -> V + 'task>(f: F) -> Self {
		Self {
			result: None,
			calculation: Box::new(f),
		}
	}

	/// Get a mutable reference to the task's output.
	pub fn get(
		&mut self,
		cache: &BTreeMap<&str, V>,
	) -> Result<&mut V, error::ExecuteError> {
		match self.result {
			Some(ref mut res) => Ok(res),
			None => {
				let v = self.calculation.execute(cache)?;
				Ok(self.result.get_or_insert(v))
			}
		}
	}

	/// Execute the task and return the result.
	pub fn consume(
		mut self,
		cache: &BTreeMap<&str, V>,
	) -> Result<V, error::ExecuteError> {
		match self.result {
			Some(res) => Ok(res),
			None => self.calculation.execute(cache),
		}
	}

	/// Get this task's dependencies.
	pub fn dependencies(&self) -> &'static [&'static str] {
		self.calculation.dependencies()
	}
}
