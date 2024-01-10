use crate::{Cache, ExecuteError, Task};
use std::collections::{BTreeMap, HashSet};

/// A DSK is a directed acyclic graph of Tasks.
/// It is used to execute a series of tasks in a specific order.
/// The order is determined by the dependencies of each task.
#[derive(Debug)]
pub struct DSK<'tasks, O>(pub(crate) BTreeMap<&'tasks str, Cache<'tasks, O>>);

impl<'tasks, O> DSK<'tasks, O> {
    /// Generates a new DSK
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Adds a task to the DSK
    pub fn add_task<T: Task<O> + 'tasks>(
        &mut self,
        key: &'tasks str,
        task: T,
    ) -> Result<(), ExecuteError> {
        if self.0.contains_key(key) {
            return Err(ExecuteError::TaskAlreadyExists(key.to_string()));
        } else {
            self.0.insert(key, Cache::from(task));
        }

        self.check_cyclic_dependencies(key)
    }

    /// Culls any tasks that are not useful in resolving the provided tasks
    pub fn cull(mut self, keys: &[&str]) -> Result<DSK<'tasks, O>, ExecuteError> {
        let mut required = HashSet::new();

        fn read_dependencies<'tasks, O>(
            dsk: &DSK<'tasks, O>,
            key: &'tasks str,
            required: &mut HashSet<&'tasks str>,
        ) -> Result<(), ExecuteError> {
            if required.contains(key) {
                return Ok(());
            }

            match dsk.0.get(key) {
                Some(task) => {
                    required.insert(key);
                    for dep in task.dependencies() {
                        read_dependencies(dsk, dep, required)?;
                    }
                }
                None => return Err(ExecuteError::MissingDependency(key.to_string())),
            }

            Ok(())
        }

        for task in keys {
            read_dependencies(&self, task, &mut required)?;
        }

        self.0.retain(|key, _| required.contains(key));
        Ok(self)
    }

    /// Returns a Set of all the keys in the DSK, that are also in the slice provided
    pub fn keys_in_dsk(&self, tasks: &[&dyn Task<O>]) -> HashSet<&'tasks str> {
        let tasks_iter = tasks
            .into_iter()
            .flat_map(|t| t.dependencies().into_iter())
            .collect::<HashSet<_>>();
        self.0
            .keys()
            .filter(|k| tasks_iter.contains(*k))
            .cloned()
            .collect()
    }

    /// Gives us a Set of direct dependencies
    pub fn get_dependents(&self, leaf: &str) -> HashSet<&'tasks str> {
        self.0
            .iter()
            .filter(|(_, task)| task.dependencies().contains(&leaf))
            .map(|(name, _)| *name)
            .collect()
    }

    /// This checks for cyclic dependencies in the graph during task insertion.
    pub fn check_cyclic_dependencies(&self, root: &str) -> Result<(), ExecuteError> {
        let mut visited = HashSet::new();
        let mut stack = Vec::with_capacity(self.0.len() / 2);

        fn find_cycles<'root, O>(
            dsk: &DSK<'root, O>,
            root: &'root str,
            visited: &mut HashSet<&'root str>,
            stack: &mut Vec<&'root str>,
        ) -> Result<(), ExecuteError> {
            stack.push(root);

            if visited.contains(root) {
                return Err(ExecuteError::CyclicDependency(
                    stack.into_iter().map(|f| f.to_string()).collect(),
                ));
            }

            visited.insert(root);

            if let Some(t) = dsk.0.get(root) {
                for dep in t.dependencies() {
                    find_cycles(dsk, dep, visited, stack)?;
                    visited.clear();
                }
            }

            stack.pop();

            Ok(())
        }

        find_cycles(self, root, &mut visited, &mut stack)
    }
}

impl<'tasks, O: Clone> DSK<'tasks, O> {
    /// Executes the queried task, resolving it's dependencies and caching the result
    /// O implements Clone, so we can cache the result
    pub fn get(
        &mut self,
        task_name: &'tasks str,
        // ==+== ==+== ==+== //
        cache: &mut BTreeMap<&'tasks str, O>,
    ) -> Result<O, ExecuteError> {
        let dependencies = {
            let task = self.0.get(task_name);
            task.map(|d| d.dependencies())
                .ok_or(ExecuteError::MissingDependency(task_name.to_string()))?
        };

        for dep in dependencies {
            self.get(dep, cache)?;
        }

        let map = &mut self.0;
        let task = map
            .get_mut(task_name)
            .ok_or(ExecuteError::MissingDependency(task_name.to_string()))?;
        task.get(cache).cloned()
    }
}

impl<'tasks, O, I, T> std::iter::FromIterator<(I, T)> for DSK<'tasks, O>
where
    I: Into<&'tasks str>,
    T: Task<O> + 'tasks,
{
    /// Generates a new DSK from an Iterator of str and Tasks
    fn from_iter<Iter: IntoIterator<Item = (I, T)>>(iter: Iter) -> Self {
        let mut dsk = Self::new();

        iter.into_iter()
            .for_each(|(key, task)| dsk.add_task(key.into(), task).unwrap());

        dsk
    }
}
