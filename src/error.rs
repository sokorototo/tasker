#[derive(Debug, thiserror::Error)]
/// Any error encountered during task insertion or evaluation
pub enum ExecuteError {
	/// The specified task does not exist in the DSK
	#[error("Key {0} is not a key in the graph")]
	MissingDependency(String),
	/// A NullTask is used for structural purposes. It should never be executed.
	#[error("Attempted to execute a NULL Task")]
	NullTask,
	/// A circular dependency chain was detected
	#[error("A circular dependency chain: (0:?)was detected")]
	CyclicDependency(Vec<String>),
	/// Tried to insert a Task into a slot that already has a Task. Which would overwrite the existing Task.c
	#[error("A Task with the key: {0} already exists")]
	TaskAlreadyExists(String),
}
