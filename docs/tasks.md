# Kernel Task and Execution Model

## Scope

The task subsystem began in `v0.0.6` as identity/state storage. During `v0.1.0` development it gained task-owned execution state, scheduler integration, cooperative context switching, and now a verified IRQ-driven Ring 0 preemption path.

The current execution model has three explicit layers:

1. `kernel::task` — task identity, state, saved cooperative/preemptive context, and kernel-stack ownership metadata;
2. `kernel::scheduler` — round-robin policy and resolution of switch decisions to task-owned contexts;
3. `arch::context` / `arch::preemption` — x86_64 register/stack save and restore mechanisms.

Policy stays in `kernel`; privileged stack/register manipulation stays in `arch`.

## Task contract

`TaskTable` stores up to 16 tasks. Each `Task` currently owns or can own:

- opaque monotonic `TaskId`;
- `TaskState`: `Ready`, `Running`, `Blocked`, or `Terminated`;
- cooperative `arch::context::Context` state;
- preemptive `arch::preemption::PreemptionContext` state;
- optional `KernelStack` metadata containing the stack start and length.

`Task` is intentionally not `Copy`. Execution ownership must remain attached to a stable task object rather than silently duplicating saved CPU state.

The bootstrap task is created during boot and starts as `Running`. A worker task is then created with an independent 4 KiB physical-frame-backed kernel stack and a prepared interrupt-return frame.

## Scheduler contract

`Scheduler` owns the persistent `TaskTable` and implements a round-robin policy with a default quantum of five PIT ticks.

`Scheduler::on_tick()` returns one of:

```text
Continue(task)
Switch { from, to }
Idle
```

Two architecture mechanisms can consume a switch decision during the current development history:

- `prepare_context_switch()` resolves a cooperative switch into saved `Context` objects;
- `prepare_preemption_switch()` stores the live interrupted frame as the outgoing task's `PreemptionContext` and returns the selected incoming frame pointer.

The scheduler does **not** execute architecture assembly itself. The timer entry/return mechanism is owned by `arch`; scheduler policy only decides which task should run.

## Saved x86_64 contexts

### Cooperative context

The earlier cooperative architecture context stores `rsp` and preserves the SysV callee-saved register set:

- `rbp`;
- `rbx`;
- `r12`;
- `r13`;
- `r14`;
- `r15`.

Fresh cooperative contexts use a trampoline to transfer control to a non-returning task entry function.

### Preemption context

IRQ-driven switching uses a wider stack-resident `InterruptContext`:

```text
r15 r14 r13 r12 r11 r10 r9 r8
rbp rdi rsi rdx rcx rbx rax
RIP CS RFLAGS RSP SS
```

The 15 GPR values are pushed by the raw timer entry. The five final values are the complete 64-bit CPU interrupt-return frame. `iretq` consumes that frame when the selected task resumes.

Fresh preemptive kernel tasks are built in exactly this representation. Their synthetic `RSP` points to a SysV-compatible entry stack and their synthetic `SS` uses the kernel data selector.

## Runtime preemption proof

The reference QEMU runtime uses real 100 Hz PIT interrupts.

Current sequence:

```text
bootstrap task (Running)
        |
        | fifth PIT tick enters IRQ0
        v
scheduler selects worker
        |
        | iretq into worker's prepared frame
        v
worker task spins forever without yielding
        |
        | fifth PIT tick preempts worker
        v
scheduler saves worker frame and selects bootstrap
        |
        v
bootstrap resumes
        |
        | repeated until four IRQ-driven switches
        v
preemption proof completes
        |
        v
Ring 3 syscall/breakpoint probe
```

CI requires:

```text
Preemption: worker executed without yielding.
Preemption: four IRQ-driven task switches observed.
Preemption: bootstrap resumed twice.
Ring 3 syscall entered kernel and returned.
Ring 3 probe reached kernel breakpoint.
```

CI #144 passed formatting, host tests, `cargo check`, Clippy, target build, and this QEMU/OVMF smoke path after the complete interrupt frame was fixed.

## Stable runtime ownership

The current early runtime places the `Scheduler` in a dedicated physical frame accessed through the bootloader physical-memory mapping. This gives the scheduler and its owned `TaskTable` a stable address while execution moves between stacks.

The worker stack is another allocated physical frame. The same monotonic `FrameAllocator` instance is used for scheduler state, worker stack, and later Ring 3/page-table allocations so the same physical frame cannot be handed out twice by independent allocator cursors.

This is still an early allocation model: frames are not yet reclaimed and there is no general task-stack allocator/teardown subsystem.

## Important userspace-preemption boundary

Ring 0 preemption is verified. Scheduler-managed Ring 3 preemption is intentionally **not enabled yet**.

The current TSS uses one shared Ring 0 privilege stack. A timer interrupt from Ring 3 would place the saved user frame on that privilege stack. If another user task were then run and interrupted using the same TSS `RSP0`, it could overwrite the older suspended task's saved frame.

The next privilege-boundary work must therefore make these lifetimes explicit:

```text
User Task
   |
   +--> user execution stack
   +--> saved user InterruptContext
   +--> owned Ring 0 privilege/interrupt stack
              |
              +--> selected as TSS RSP0 while that task runs in Ring 3
```

Only after the scheduler and TSS selection are connected this way should the timer preemption hook remain active during general Ring 3 execution.

## Current limitations

- Maximum of 16 task-table entries.
- Kernel stacks use early physical-frame allocation with no reclaim path.
- The preemption proof currently switches Ring 0 tasks only.
- The TSS Ring 0 privilege stack is still shared rather than task-owned.
- No task priorities, affinity, wait queues, sleep/wakeup mechanism, or SMP synchronization.
- No process/thread relationship or per-task address space.
- `Terminated` tasks do not yet have full reaping/resource reclamation.
- There is no first persistent userspace `init` or shell yet.

## Regression coverage

Host tests cover:

- monotonic task IDs and initial state;
- state transitions and capacity errors;
- single-running-task invariant;
- task ownership of cooperative/preemptive contexts and `KernelStack` metadata;
- round-robin decisions;
- resolving `Switch { from, to }` to owned execution contexts;
- saving the interrupted frame and selecting a task-owned preemption frame;
- fresh complete interrupt-frame construction, including `RSP`/`SS`;
- kernel selector privilege levels;
- bootstrap task-table construction;
- cooperative fresh-context stack preparation/alignment.

QEMU covers:

- real PIT IRQs entering scheduler runtime;
- a worker body that executes without yielding;
- repeated IRQ-driven bootstrap/worker switches on separate stacks;
- continued Ring 3 syscall entry/return;
- final Ring 3 breakpoint entry into the kernel;
- absence of kernel panic/triple fault during the smoke path.

## Next step

The next scheduler slice is **per-task Ring 0 privilege-stack ownership and TSS `RSP0` selection**, followed by a scheduler-owned Ring 3 task whose execution can be interrupted and resumed repeatedly.

After that is stable, `v0.1.0` still requires userspace `init`, a minimal shell, syscall integration for those programs, and basic termination/lifecycle behavior before the milestone can be considered complete.
