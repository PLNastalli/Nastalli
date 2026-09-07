## Resumo

<!-- O que mudou e qual problema concreto isso resolve? -->

## Verificação

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo xtask test`
- [ ] `cargo check -p nastalli-arch -p nastalli-hal -p nastalli-kernel -p xtask`
- [ ] `cargo xtask build`
- [ ] `cargo xtask run` (quando a mudança afetar boot ou hardware)

## Segurança e arquitetura

- [ ] Documentação atualizada quando necessário.
- [ ] Todo `unsafe` novo tem justificativa.
- [ ] Não introduzi dependência direta de hardware no kernel sem razão documentada.
- [ ] Limitações ou riscos conhecidos estão descritos abaixo.

## Notas

<!-- Inclua saída relevante, limitações, riscos ou próximos passos. -->
