# Kernel Task Model — v0.0.6

## Scope

`v0.0.6` introduces only the identity, state, and ownership model for kernel tasks.

It does **not** yet implement:

- scheduling policy;
- context switching;
- independent task stacks;
- processes;
- userspace execution;
- preemption.

## Current contract

`kernel::task::TaskTable` stores up to 16 entries in a fixed-size table.

Each task contains:

- an opaque, monotonic `TaskId` backed by `u64`;
- a `TaskState` value: `Ready`, `Running`, `Blocked`, or `Terminated`.

The table currently supports explicit task creation, lookup, and state transition.

The first task created during boot represents the bootstrap task and is marked `Running` as a descriptive state only. No scheduler currently decides which task receives CPU time.

## Lifetime and ownership

The task table must survive the initialization function that creates it.

For that reason, the boot-time task initialization path returns the `TaskTable`, and ownership is moved into the kernel's long-running runtime loop.

Conceptually:

```text
initialize_tasks()
      |
      v
   TaskTable
      |
      | move ownership
      v
kernel runtime
```

This avoids introducing a global/static task database before the architecture has a real requirement for globally synchronized scheduler state.

The runtime currently keeps the table alive but does not schedule, block, wake, preempt, or switch between tasks.

## Design decisions

- The fixed capacity avoids introducing dynamic task-storage policy while virtual memory and scheduling are still being built.
- The table is an internal kernel structure, not a public ABI.
- Handles and capabilities are deferred until syscall/resource semantics exist.
- The task model does not depend on x86_64 register or stack types, preserving architecture independence at this layer.
- Long-lived state has explicit ownership rather than an early singleton.

## Current limitations

- Maximum of 16 table entries.
- A terminated task does not yet have a full reaping/resource-reclamation lifecycle.
- `Task` is currently small and copyable; future context/address-space ownership will likely require reference-based or otherwise non-copying access patterns.
- There is no priority model, CPU affinity, runtime accounting, wait queue, or task-local address space.
- There is no synchronization model for SMP.

These limitations should be addressed when the scheduler/process model creates concrete requirements for them.

## Regression coverage

The kernel includes a regression test that validates construction of the bootstrap task state produces exactly one task.

`TaskTable` tests also cover:

- monotonic task IDs;
- initial `Ready` state;
- explicit state updates;
- capacity exhaustion;
- missing-task lookup behavior.

## Next milestone

`v0.0.7` is planned to introduce an initial scheduler policy that consumes timer ticks while preserving the existing persistent task state.

Context representation, kernel stacks, and actual context switching should be introduced incrementally and documented as their invariants become concrete.
