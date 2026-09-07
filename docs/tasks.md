# Kernel Task and Execution Model

## Scope

The task subsystem started in `v0.0.6` as identity/state storage. Since then, Nastalli has added scheduler policy and a verified architecture-level context-switch primitive, but those pieces are not yet fully integrated into a production task lifecycle.

The current state is intentionally split into three layers:

1. `kernel::task` — task identity and descriptive state;
2. `kernel::scheduler` — round-robin scheduling policy driven by PIT tick accounting;
3. `arch::context` — x86_64 saved-context preparation and low-level context switching.

A dedicated QEMU probe now proves that the third layer can execute on an independent kernel stack and resume both contexts correctly. Scheduler decisions do **not** yet invoke that mechanism.

## Task contract

`kernel::task::TaskTable` stores up to 16 entries in a fixed-size table.

Each task contains:

- an opaque, monotonic `TaskId` backed by `u64`;
- a `TaskState` value: `Ready`, `Running`, `Blocked`, or `Exited`.

The table supports explicit task creation, lookup, and state transition.

The first task created during boot represents the bootstrap task and is marked `Running`. The table is moved into the long-lived scheduler so ownership survives initialization.

## Scheduler policy

`kernel::scheduler::Scheduler` owns the persistent `TaskTable` and implements the current round-robin policy.

Current behavior:

- real PIT IRQ0 ticks are counted at 100 Hz;
- the default quantum is 5 ticks;
- runnable task states can be rotated by `Scheduler::on_tick()`;
- scheduler tests validate the policy and state transition behavior.

A `ScheduleDecision::Switch { from, to }` currently expresses policy only. It does not yet trigger `arch::context::switch` or move the CPU to a task-owned stack.

This distinction is important: timer-driven **scheduling decisions** exist, but timer-driven **execution-context switching** does not yet exist.

## Saved kernel context

`arch::context::Context` currently contains the saved kernel stack pointer:

```text
Context
└── stack_pointer: u64
```

The architecture switch routine stores/restores the x86_64 SysV callee-saved register context:

- `rbp`;
- `rbx`;
- `r12`;
- `r13`;
- `r14`;
- `r15`;
- `rsp` through `Context::stack_pointer`.

Fresh contexts are prepared with an initial saved-register frame and an architecture trampoline. The trampoline passes the prepared argument to the supplied non-returning entry function.

Raw stack construction and register manipulation live in `crates/arch`, not scheduler policy.

## Independent-stack validation

During `v0.1.0` development, the kernel performs a narrow cooperative validation before entering the existing Ring 3 probe.

The validation:

1. allocates one 4 KiB usable physical frame from the runtime `FrameAllocator`;
2. accesses it through the bootloader's direct physical-memory mapping;
3. places stable saved-context state at the beginning of that page;
4. uses the rest of the page as an independent worker kernel stack;
5. switches bootstrap -> worker;
6. switches worker -> bootstrap;
7. switches bootstrap -> the previously suspended worker;
8. switches worker -> bootstrap again;
9. continues into the Ring 3 syscall probe.

The CI smoke test requires these runtime markers in order:

```text
Context switch: worker entered.
Context switch: bootstrap resumed.
Context switch: worker resumed.
Context switch: bootstrap resumed twice.
```

The same boot must subsequently reach:

```text
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

This proves actual execution continuation across two kernel stacks rather than only scheduler-state mutation.

## Frame ownership during the validation path

The context worker stack and the later Ring 3 code/stack/page-table frames use one shared monotonic `FrameAllocator` instance.

This matters because constructing multiple independent allocators over the same immutable boot memory map would allow each allocator to hand out the same first usable frames. The runtime instead keeps one allocator cursor for all of these early consumers.

This is still an early boot-time ownership model, not a general physical-memory manager capable of freeing frames.

## What `Task` does not own yet

Despite the verified context-switch mechanism, an individual `Task` still does **not** own:

- a `Context`;
- a kernel stack allocation;
- an address space;
- credentials or capabilities;
- a process/thread relationship;
- runtime accounting;
- wait-queue membership;
- resource teardown state.

This is the main integration gap for the next scheduler work.

## Lifetime and ownership

Long-lived runtime state must have a stable owner.

For the normal task model:

```text
initialize_tasks()
      |
      v
   TaskTable
      |
      v
  Scheduler
      |
      v
kernel runtime
```

For the current context-switch probe, saved bootstrap/worker contexts live at a stable address in the dedicated worker frame while either stack may be inactive. This avoids depending on a temporary object whose lifetime or location could become invalid across a stack switch.

The production task integration must generalize this rule rather than keep probe-specific state.

## Current limitations

- Maximum of 16 task-table entries.
- `Task` still stores only identity/state and remains copyable.
- Context and kernel-stack lifecycle are not yet integrated into `Task`.
- `ScheduleDecision::Switch` does not yet perform a CPU switch.
- No PIT interrupt performs a context switch.
- Multiple task bodies are not yet preemptively executed.
- There is no generic kernel-stack allocator/free path.
- Exited tasks do not yet have full reaping/resource reclamation.
- There is no priority, affinity, wait queue, task-local address space, or SMP synchronization model.
- Process/thread semantics are not implemented.

## Regression and runtime coverage

Host tests cover:

- monotonic task IDs;
- initial `Ready` state;
- explicit state updates;
- capacity exhaustion;
- missing-task lookup behavior;
- scheduler round-robin decisions;
- bootstrap task-table construction;
- fresh-context stack preparation/alignment.

QEMU smoke validation covers:

- first real PIT tick reaching scheduler runtime;
- two cooperative kernel-context round trips on an independent stack;
- continued Ring 3 syscall entry/return after those switches;
- final Ring 3 breakpoint transition back to the kernel.

## Next integration step

The next scheduler slice should move context/stack ownership into a task runtime structure and make scheduler-selected tasks execute through the verified context-switch mechanism.

Only after that path is stable should PIT-driven preemption switch live task contexts. This keeps timer/interrupt complexity separate from basic ownership and cooperative switching while the invariants are still being established.
