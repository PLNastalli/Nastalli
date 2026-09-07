use crate::task::{TaskId, TaskState, TaskTable, TaskTableError};
use nastalli_arch::context::Context;

pub const DEFAULT_QUANTUM_TICKS: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDecision {
    Continue(TaskId),
    Switch { from: TaskId, to: TaskId },
    Idle,
}

pub struct PreparedContextSwitch {
    pub current: *mut Context,
    pub next: Context,
}

pub struct Scheduler {
    tasks: TaskTable,
    current: Option<TaskId>,
    ticks_in_slice: u64,
    quantum_ticks: u64,
}

impl Scheduler {
    pub fn new(tasks: TaskTable) -> Self {
        Self::with_quantum(tasks, DEFAULT_QUANTUM_TICKS)
    }

    pub fn with_quantum(mut tasks: TaskTable, quantum_ticks: u64) -> Self {
        assert!(quantum_ticks > 0, "scheduler quantum must be non-zero");

        let current = tasks.running_task().or_else(|| {
            let next = tasks.next_ready_after(None)?;
            tasks
                .set_state(next, TaskState::Running)
                .expect("ready task can become running");
            Some(next)
        });

        Self {
            tasks,
            current,
            ticks_in_slice: 0,
            quantum_ticks,
        }
    }

    pub const fn current(&self) -> Option<TaskId> {
        self.current
    }

    pub fn task_state(&self, id: TaskId) -> Result<TaskState, TaskTableError> {
        self.tasks.get(id).map(|task| task.state())
    }

    pub fn on_tick(&mut self) -> ScheduleDecision {
        let Some(current) = self.current else {
            return ScheduleDecision::Idle;
        };

        self.ticks_in_slice = self.ticks_in_slice.saturating_add(1);
        if self.ticks_in_slice < self.quantum_ticks {
            return ScheduleDecision::Continue(current);
        }
        self.ticks_in_slice = 0;

        let Some(next) = self.tasks.next_ready_after(Some(current)) else {
            return ScheduleDecision::Continue(current);
        };

        self.tasks
            .set_state(current, TaskState::Ready)
            .expect("current task exists");
        self.tasks
            .set_state(next, TaskState::Running)
            .expect("ready task can become running");
        self.current = Some(next);

        ScheduleDecision::Switch {
            from: current,
            to: next,
        }
    }

    pub fn prepare_context_switch(
        &mut self,
        decision: ScheduleDecision,
    ) -> Result<PreparedContextSwitch, TaskTableError> {
        let ScheduleDecision::Switch { from, to } = decision else {
            panic!("context switch preparation requires a switch decision");
        };

        let next = self.tasks.context(to)?;
        let current = self.tasks.context_mut(from)? as *mut Context;

        Ok(PreparedContextSwitch { current, next })
    }
}

#[cfg(test)]
mod tests {
    use super::{ScheduleDecision, Scheduler};
    use crate::task::{KernelStack, TaskState, TaskTable};
    use nastalli_arch::context::Context;

    #[test]
    fn rotates_ready_tasks_when_the_quantum_expires() {
        let mut tasks = TaskTable::new();
        let first = tasks.create().unwrap();
        let second = tasks.create().unwrap();
        tasks.set_state(first, TaskState::Running).unwrap();

        let mut scheduler = Scheduler::with_quantum(tasks, 2);

        assert_eq!(scheduler.on_tick(), ScheduleDecision::Continue(first));
        assert_eq!(
            scheduler.on_tick(),
            ScheduleDecision::Switch {
                from: first,
                to: second,
            }
        );
        assert_eq!(scheduler.task_state(first).unwrap(), TaskState::Ready);
        assert_eq!(scheduler.task_state(second).unwrap(), TaskState::Running);
    }

    #[test]
    fn prepares_owned_contexts_for_a_switch_decision() {
        let mut tasks = TaskTable::new();
        let first = tasks.create().unwrap();
        let second = tasks.create().unwrap();
        tasks
            .install_execution(
                first,
                Context {
                    stack_pointer: 0x1111,
                },
                KernelStack::new(0x1000, 4096),
            )
            .unwrap();
        tasks
            .install_execution(
                second,
                Context {
                    stack_pointer: 0x2222,
                },
                KernelStack::new(0x2000, 4096),
            )
            .unwrap();
        tasks.set_state(first, TaskState::Running).unwrap();

        let mut scheduler = Scheduler::with_quantum(tasks, 1);
        let decision = scheduler.on_tick();
        let prepared = scheduler.prepare_context_switch(decision).unwrap();

        assert_eq!(prepared.next.stack_pointer, 0x2222);
        assert_eq!(unsafe { (*prepared.current).stack_pointer }, 0x1111);
    }
}
