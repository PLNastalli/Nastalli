# Contribuindo com o Nastalli OS

Obrigado pelo interesse. O Nastalli está em estágio inicial; mudanças pequenas, verificáveis e bem documentadas são mais valiosas que grandes abstrações prematuras.

## Antes de abrir uma mudança

- Leia [docs/architecture.md](docs/architecture.md), [docs/unsafe-policy.md](docs/unsafe-policy.md) e [SECURITY.md](SECURITY.md).
- Explique o problema concreto que a mudança resolve.
- Preserve as fronteiras entre `arch`, `hal` e `kernel`.
- Não adicione crates ou arquivos vazios sem uma responsabilidade real.

## Verificação local

```bash
cargo fmt --all -- --check
cargo xtask test
cargo check -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask
cargo xtask build
```

Se a mudança envolver boot, também execute `cargo xtask run` no QEMU e registre a saída serial em `docs/progress.md`.

## Commits e pull requests

Use mensagens curtas e descritivas, por exemplo `feat: initialize task table` ou `docs: clarify memory ownership`. Um pull request deve explicar o motivo, os riscos, a verificação executada e as limitações conhecidas.

Mudanças em segurança, boot, memória, interrupções ou `unsafe` precisam de revisão especialmente cuidadosa.
