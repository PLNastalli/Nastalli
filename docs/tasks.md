# Tarefas do kernel

## Estado da v0.0.6

A v0.0.6 introduz somente o modelo de identidade e estado de tarefas. Não existe scheduler, troca de contexto, stack própria, processo, userspace ou preempção.

## Contrato atual

`kernel::task::TaskTable` mantém até 16 entradas em uma tabela fixa. Cada entrada possui:

- `TaskId` monotônico, opaco e representável como `u64`;
- `TaskState`: `Ready`, `Running`, `Blocked` ou `Terminated`;
- operações explícitas de criação, consulta e mudança de estado.

O primeiro item criado durante o boot representa a tarefa bootstrap e passa para `Running` apenas como estado descritivo. Nenhum código ainda decide qual tarefa recebe CPU.

## Decisões

- A capacidade fixa evita depender de uma política de alocação dinâmica antes da paginação do kernel.
- A tabela fica no kernel e não é uma ABI pública; handles e capabilities serão definidos somente quando existirem syscalls.
- O módulo não conhece registradores, stacks ou detalhes de `x86_64`, preservando a possibilidade de ARM64.

## Próximo passo

A v0.0.7 poderá consumir os ticks do PIT e definir uma política de scheduler. Antes disso, será necessário decidir como representar contexto salvo, stack de kernel e sincronização entre CPUs.
