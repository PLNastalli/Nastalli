# Tarefas do kernel

## Estado da v0.0.6

A v0.0.6 introduz somente o modelo de identidade e estado de tarefas. Não existe scheduler, troca de contexto, stack própria, processo, userspace ou preempção.

## Contrato atual

`kernel::task::TaskTable` mantém até 16 entradas em uma tabela fixa. Cada entrada possui:

- `TaskId` monotônico, opaco e representável como `u64`;
- `TaskState`: `Ready`, `Running`, `Blocked` ou `Terminated`;
- operações explícitas de criação, consulta e mudança de estado.

O primeiro item criado durante o boot representa a tarefa bootstrap e passa para `Running` apenas como estado descritivo. Nenhum código ainda decide qual tarefa recebe CPU.

## Vida útil e ownership

A tabela de tarefas precisa sobreviver ao fim da rotina de inicialização. Por isso, `initialize_tasks()` devolve a `TaskTable` e transfere sua posse para o runtime longo do kernel (`run`). Isso evita uma global estática prematura e deixa explícito quem é dono do estado que o scheduler da v0.0.7 irá consumir.

O runtime atual apenas mantém a tabela viva; ele ainda não agenda, bloqueia, acorda ou troca contexto entre tarefas.

## Decisões

- A capacidade fixa evita depender de uma política de alocação dinâmica antes da paginação do kernel.
- A tabela fica no kernel e não é uma ABI pública; handles e capabilities serão definidos somente quando existirem syscalls.
- O módulo não conhece registradores, stacks ou detalhes de `x86_64`, preservando a possibilidade de ARM64.
- O estado de tarefas não usa singleton/global nesta fase; o ownership permanece explícito no fluxo do kernel.

## Verificação de regressão

Existe um teste no crate do kernel que valida que a construção do estado bootstrap produz exatamente uma tarefa. Os testes unitários do próprio `TaskTable` continuam cobrindo IDs monotônicos, estados, capacidade e lookup inexistente.

## Próximo passo

A v0.0.7 poderá consumir os ticks do PIT e definir uma política de scheduler usando a mesma tabela que sobrevive ao boot. Antes de troca de contexto real, ainda será necessário decidir como representar contexto salvo, stack de kernel e sincronização entre CPUs.
