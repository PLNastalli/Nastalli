//! Minimal kernel task structures.
//!
//! This version models task identity and state while the scheduler is being
//! introduced. The fixed-size table avoids depending on a dynamic task-storage
//! policy while virtual memory and context switching are still being built.

pub const MAX_TASKS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(u64);

impl TaskId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Task {
    id: TaskId,
    state: TaskState,
}

impl Task {
    pub const fn id(self) -> TaskId {
        self.id
    }

    pub const fn state(self) -> TaskState {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskTableError {
    Capacity,
    NotFound,
    RunningTaskExists,
}

pub struct TaskTable {
    entries: [Option<Task>; MAX_TASKS],
    next_id: u64,
}

impl TaskTable {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_TASKS],
            next_id: 0,
        }
    }

    pub fn create(&mut self) -> Result<TaskId, TaskTableError> {
        let slot = self
            .entries
            .iter_mut()
            .find(|entry| entry.is_none())
            .ok_or(TaskTableError::Capacity)?;
        let id = TaskId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(TaskTableError::Capacity)?;
        *slot = Some(Task {
            id,
            state: TaskState::Ready,
        });
        Ok(id)
    }

    pub fn get(&self, id: TaskId) -> Result<Task, TaskTableError> {
        self.entries
            .iter()
            .flatten()
            .find(|task| task.id == id)
            .copied()
            .ok_or(TaskTableError::NotFound)
    }

    pub fn set_state(&mut self, id: TaskId, state: TaskState) -> Result<(), TaskTableError> {
        if state == TaskState::Running
            && self
                .entries
                .iter()
                .flatten()
                .any(|task| task.id != id && task.state == TaskState::Running)
        {
            return Err(TaskTableError::RunningTaskExists);
        }

        let task = self
            .entries
            .iter_mut()
            .flatten()
            .find(|task| task.id == id)
            .ok_or(TaskTableError::NotFound)?;
        task.state = state;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn running_task(&self) -> Option<TaskId> {
        self.entries
            .iter()
            .flatten()
            .find(|task| task.state == TaskState::Running)
            .map(|task| task.id)
    }

    pub(crate) fn next_ready_after(&self, current: Option<TaskId>) -> Option<TaskId> {
        let start = current
            .and_then(|id| {
                self.entries
                    .iter()
                    .position(|entry| entry.is_some_and(|task| task.id == id))
            })
            .map(|index| (index + 1) % MAX_TASKS)
            .unwrap_or(0);

        for offset in 0..MAX_TASKS {
            let index = (start + offset) % MAX_TASKS;
            if let Some(task) = self.entries[index] {
                if task.state == TaskState::Ready {
                    return Some(task.id);
                }
            }
        }

        None
    }
}

impl Default for TaskTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{KernelStack, MAX_TASKS, TaskState, TaskTable, TaskTableError};
    use nastalli_arch::context::Context;

    #[test]
    fn creates_ready_tasks_with_monotonic_ids() {
        let mut table = TaskTable::new();
        let first = table.create().unwrap();
        let second = table.create().unwrap();

        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        assert_eq!(table.get(first).unwrap().state(), TaskState::Ready);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn updates_state_without_scheduler_side_effects() {
        let mut table = TaskTable::new();
        let id = table.create().unwrap();

        table.set_state(id, TaskState::Running).unwrap();

        assert_eq!(table.get(id).unwrap().state(), TaskState::Running);
    }

    #[test]
    fn rejects_a_second_running_task() {
        let mut table = TaskTable::new();
        let first = table.create().unwrap();
        let second = table.create().unwrap();

        table.set_state(first, TaskState::Running).unwrap();

        assert_eq!(
            table.set_state(second, TaskState::Running),
            Err(TaskTableError::RunningTaskExists)
        );
    }

    #[test]
    fn reports_capacity_and_missing_tasks() {
        let mut table = TaskTable::new();
        for _ in 0..MAX_TASKS {
            table.create().unwrap();
        }

        assert_eq!(table.create(), Err(TaskTableError::Capacity));
        assert_eq!(table.get(super::TaskId(999)), Err(TaskTableError::NotFound));
    }

    #[test]
    fn task_owns_execution_context_and_kernel_stack() {
        let mut table = TaskTable::new();
        let id = table.create().unwrap();

        assert_eq!(table.get(id).unwrap().context(), Context::empty());
        assert_eq!(table.get(id).unwrap().kernel_stack(), None);

        let context = Context {
            stack_pointer: 0x1234,
        };
        let stack = KernelStack::new(0x8000, 4096);
        table.install_execution(id, context, stack).unwrap();

        let task = table.get(id).unwrap();
        assert_eq!(task.context(), context);
        assert_eq!(task.kernel_stack(), Some(stack));
    }
}
