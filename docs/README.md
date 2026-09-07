# Documentação da Nastalli OS

Esta documentação é parte do projeto, não um texto separado do código. Ela deve explicar o estado real do sistema, as decisões que levaram a ele, suas limitações e como reproduzir os resultados.

## Documentos

| Documento | Conteúdo |
|---|---|
| [architecture.md](architecture.md) | Camadas, dependências e fronteiras atuais |
| [boot.md](boot.md) | Cadeia UEFI → bootloader → kernel e comandos de execução |
| [progress.md](progress.md) | Diário técnico das versões, mudanças e evidências |
| [roadmap.md](roadmap.md) | Fases planejadas e critérios de entrada/saída |
| [unsafe-policy.md](unsafe-policy.md) | Regras para código unsafe e assembly |
| [security-model.md](security-model.md) | Posse do dispositivo, confiança, chaves e limites de segurança |
| [input.md](input.md) | Teclado PS/2 e fluxo de input |
| [tasks.md](tasks.md) | Modelo inicial de tarefas e limites da v0.0.6 |
| [superpowers/specs/2026-09-06-novaos-v001-design.md](superpowers/specs/2026-09-06-novaos-v001-design.md) | Design aprovado da fundação v0.0.1 |

## Como manter

Em cada mudança relevante, atualizar:

1. a versão/estado em `README.md`;
2. a decisão arquitetural afetada em `architecture.md`;
3. o registro cronológico em `progress.md`;
4. o documento específico da área, quando houver;
5. os comandos e resultados de verificação.

Não registrar como concluído algo que não foi compilado ou executado. Limitações do ambiente devem aparecer explicitamente.
