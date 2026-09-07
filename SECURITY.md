# Segurança do Nastalli OS

## Escopo

Este projeto é experimental e ainda não oferece garantias de segurança para uso real. O código atual inicializa hardware em QEMU e não implementa usuários, criptografia de disco, Secure Boot, capabilities ou isolamento completo.

## Princípios

- O proprietário deve controlar as chaves e a política de confiança do dispositivo.
- Não haverá telemetria obrigatória, chave mestra remota ou conta online necessária para inicializar o sistema.
- `unsafe` deve permanecer pequeno, documentado e concentrado nas fronteiras de hardware e boot.
- Vulnerabilidades não devem ser publicadas com exploits funcionais antes de existir correção ou coordenação adequada.

## Relato privado

Para uma vulnerabilidade que possa afetar usuários, não abra uma issue pública com detalhes exploráveis. Use os recursos de contato privado do perfil/repositório do GitHub ou solicite ao mantenedor um canal seguro. Inclua versão/commit, arquitetura, ambiente, passos para reproduzir com segurança e impacto observado.

Issues comuns de documentação ou bugs não sensíveis podem ser abertas publicamente.
