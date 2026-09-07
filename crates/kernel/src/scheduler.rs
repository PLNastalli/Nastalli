#[cfg(test)]
mod tests {
    use super::{ScheduleDecision, Scheduler};
    use crate::task::{TaskState, TaskTable};

    #[test]
    fn rotates_ready_tasks_when_the_quantum_expires() {
        let mut tasks = TaskTable::new();
        let first = tasks.create().unwrap();
        let second = tasks.create().unwrap();
        tasks.set_state(first, TaskState::Running).unwrap();

        let mut scheduler = Scheduler::with_quantum(tasks, 2);

        assert_eq!(scheduler.on_tick(), ScheduleDecision::Continue(first));
        assert_eq!(scheduler.on_tick(), ScheduleDecision::Switch {
            from: first,
            to: second,
        });
        assert_eq!(scheduler.task_state(first).unwrap(), TaskState::Ready);
        assert_eq!(scheduler.task_state(second).unwrap(), TaskState::Running);
    }
}
