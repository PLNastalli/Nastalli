# Nova OS v0.0.1 — Design

## Objetivo

Bootar um kernel Rust `no_std` em x86_64 por UEFI, emitir diagnóstico pela serial e produzir saída visual inicial no framebuffer.

## Componentes

- `boot`: ponto de entrada e contrato `BootInfo`.
- `arch`: assembly e I/O x86_64.
- `hal`: interface segura para serial.
- `kernel`: inicialização e uso do framebuffer.
- `xtask`: build, imagem, execução e testes.

## Segurança e evolução

Todos os crates negam operações unsafe implícitas em funções unsafe. Assembly ficará apenas em `arch`; as camadas superiores não conhecerão portas, registradores ou instruções. ABI, handles e userspace serão adicionados somente quando houver comportamento real que os justifique.

O sistema seguirá o modelo owner-controlled: nenhuma chave mestra do projeto, telemetria obrigatória ou dependência de servidor remoto será criada. Secure Boot será opcional e deverá funcionar com chaves do proprietário. A v0.0.1 não implementa criptografia, mas não cria uma autoridade central que impeça sua implementação futura.

## Critério de verificação

`cargo fmt --check`, `cargo check`, `cargo clippy`, `cargo xtask build`, `cargo xtask image` e boot observado no QEMU com OVMF.
