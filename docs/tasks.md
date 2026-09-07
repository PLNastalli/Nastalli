# Kernel Task and Execution Model

## Scope

The task subsystem began in `v0.0.6` as identity/state storage. During `v0.1.0` development it now owns saved execution context and kernel-stack metadata and is connected to the round-robin scheduler and the verified x86_64 context-switch primitive.

The current execution model has three explicit layers:

1. `kernel::task` — task identity, state, saved context, and kernel-stack ownership metadata;
2. `kernel::scheduler` — round-robin policy and resolution of switch decisions to task-owned contexts;
3. `arch::context` — x86_64 stack/register save and restore.

Policy stays in `kernel`; privileged stack/register manipulation stays in `arch`.

## Task contract

`TaskTable` stores up to 16 tasks. Each `Task` currently owns:

- opaque monotonic `TaskId`;
- `TaskState`: `Ready`, `Running`, `Blocked`, or `Terminated`;
- `arch::context::Context` containing the saved kernel stack pointer;
- optional `KernelStack` metadata containing the stack start and length.

`Task` is intentionally no longer `Copy`. Execution ownership must remain attached to a stable task object rather than silently duplicating saved CPU state.

The bootstrap task is created during boot and starts as `Running`. A worker task is then created with an independent 4 KiB physical-frame-backed kernel stack and a prepared architecture context.

## Scheduler contract

`Scheduler` owns the persistent `TaskTable` and implements a round-robin policy with a default quantum of five PIT ticks.

`Scheduler::on_tick()` returns one of:

```text
Continue(task)
Switch { from, to }
Idle
```

A `Switch` decision changes scheduler-visible task state and can then be resolved through `prepare_context_switch()` into:

- a writable pointer to the current task's owned `Context`, where the low-level switch saves the outgoing `rsp`;
- a copy of the next task's saved `Context`, used to restore its stack/register state.

The scheduler does **not** execute architecture assembly itself. The kernel runtime consumes the prepared switch and calls `arch::context::switch`.

## Saved x86_64 context

The current architecture context stores `rsp` and the switch routine preserves the SysV callee-saved register set:

- `rbp`;
- `rbx`;
- `r12`;
- `r13`;
- `r14`;
- `r15`.

Fresh contexts are built on their own kernel stack with a trampoline that transfers control to a non-returning task entry function.

This is sufficient for the current cooperative kernel-task execution proof. IRQ-driven preemption will require a wider interrupted-register/trap-frame contract rather than invoking this cooperative switch directly from IRQ0.

## Runtime execution proof

The reference QEMU runtime uses real 100 Hz PIT ticks as scheduling input.

Current sequence:

```text
bootstrap task (Running)
        |
        | scheduler quantum expires
        v
worker task (Running on independent kernel stack)
        |
        | scheduler quantum expires
        v
bootstrap task resumes
        |
        | next quantum
        v
worker task resumes
        |
        | next quantum
        v
bootstrap task resumes again
        |
        v
Ring 3 syscall probe
```

CI requires:

```text
Task switch: worker entered.
Task switch: bootstrap resumed.
Task switch: worker resumed.
Task switch: bootstrap resumed twice.
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

This proves scheduler-selected task contexts actually move CPU execution between independent stacks and that the Ring 3 path still works afterward.

## Stable runtime ownership

The current early runtime places the `Scheduler` in a dedicated physical frame accessed through the bootloader physical-memory mapping. This gives the scheduler and its owned `TaskTable` a stable address while execution moves between stacks.

The worker stack is another allocated physical frame. The same monotonic `FrameAllocator` instance is used for scheduler state, worker stack, and later Ring 3/page-table allocations so the same physical frame cannot be handed out twice by independent allocator cursors.

This is still an early allocation model: frames are not yet reclaimed and there is no general task-stack allocator/teardown subsystem.

## Important preemption boundary

The current task switches are **timer-driven by tick accounting but not IRQ-preemptive**.

IRQ0 currently records the PIT tick and returns. Normal kernel execution polls that counter, feeds elapsed ticks into `Scheduler::on_tick()`, and performs the selected context switch outside interrupt context.

Calling the existing cooperative `arch::context::switch()` directly from the PIT handler would be incorrect: it would save the handler's execution context rather than the interrupted task's complete CPU state.

Real preemption therefore needs an architecture interrupt entry/return path that captures the interrupted register/trap frame, lets kernel policy select another runnable task, and restores the selected task through interrupt return.

## Current limitations

- Maximum of 16 task-table entries.
- Kernel stacks use early physical-frame allocation with no reclaim path.
- The saved cooperative context is not yet a full interrupt/preemption context.
- PIT IRQ0 does not directly switch tasks.
- No task priorities, affinity, wait queues, sleep/wakeup mechanism, or SMP synchronization.
- No process/thread relationship or per-task address space.
- `Terminated` tasks do not yet have full reaping/resource reclamation.
- There is no first persistent userspace `init` or shell yet.

## Regression coverage

Host tests cover:

- monotonic task IDs and initial state;
- state transitions and capacity errors;
- single-running-task invariant;
- task ownership of `Context` and `KernelStack`;
- round-robin decisions;
- resolving `Switch { from, to }` to owned execution contexts;
- bootstrap task-table construction;
- architecture fresh-context stack preparation/alignment.

QEMU covers:

- real PIT ticks reaching scheduler runtime;
- repeated bootstrap/worker task switches on separate stacks;
- continued Ring 3 syscall entry/return;
- final Ring 3 breakpoint entry into the kernel;
- absence of kernel panic during the smoke path.

## Next step

The next scheduler slice is a real **IRQ-driven preemption foundation**. It must model and save the CPU state interrupted by the PIT, not reuse the cooperative switch from inside the IRQ handler.

After that is stable, `v0.1.0` still requires userspace `init`, a minimal shell, syscall integration for those programs, and basic termination/lifecycle behavior before the milestone can be considered complete.
