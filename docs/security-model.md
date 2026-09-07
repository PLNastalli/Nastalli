# Modelo de segurança e posse

## Objetivo

O Nova OS deve ser controlado pelo proprietário da máquina. “Seguro” não significa que o kernel pode impedir o dono de fazer mudanças; significa que os limites de autoridade são explícitos, verificáveis e não dependem de uma autoridade remota do projeto.

## Invariantes desejadas

1. Não existe chave mestra do Nova OS capaz de desbloquear máquinas de usuários.
2. Não existe telemetria obrigatória ou conta online necessária para inicializar o sistema.
3. Chaves de Secure Boot, quando usadas, pertencem ao proprietário.
4. Dados persistentes devem poder ser criptografados com chaves sob controle do proprietário.
5. Processos futuros recebem handles/capabilities explícitos, não acesso global implícito.
6. Criptografia ponta a ponta de comunicação pertence a userspace; o kernel fornece isolamento, RNG e transporte de dados, mas não deve conhecer o conteúdo das mensagens.
7. Falhas de um driver ou serviço não devem conceder automaticamente autoridade sobre todos os recursos.

## O que existe hoje

A v0.0.2 ainda não implementa criptografia, Secure Boot, usuários ou capabilities. A contribuição de segurança atual é estrutural: `unsafe` concentrado, ausência de serviços remotos, separação de camadas e dependência explícita de um bootloader conhecido.

## Plano futuro

### Boot e confiança

- medir e documentar a cadeia de boot;
- permitir Secure Boot com chaves do proprietário;
- oferecer modo de recuperação local, sem escrow obrigatório do projeto;
- separar atualização assinada de decisão de confiança do usuário.

### Dados

- gerar chaves localmente usando fonte de aleatoriedade adequada;
- criptografar armazenamento por usuário/dispositivo;
- manter recuperação como escolha explícita do proprietário;
- evitar que logs exponham segredos.

### Isolamento

- processos e threads com espaços de memória separados;
- handles com direitos verificáveis;
- IPC explícito;
- drivers gradualmente movidos para domínios menos privilegiados quando a base permitir.

## Limites

O sistema não pode garantir segurança contra firmware malicioso, hardware adulterado, comprometimento físico completo ou um proprietário que voluntariamente execute código malicioso com privilégios totais. Essas ameaças devem ser tratadas como parte do modelo de ameaça, não escondidas por slogans de criptografia.
