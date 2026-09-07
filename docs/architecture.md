# Arquitetura inicial

`boot` contém o contrato com o bootloader e o entry point. `kernel` contém a inicialização independente do hardware. `hal` expõe o escritor serial; `arch` concentra I/O x86_64 e assembly. `xtask` centraliza os comandos de desenvolvimento.

O framebuffer é recebido por `BootInfo` e usado diretamente apenas pela inicialização gráfica mínima. Uma abstração de framebuffer só será criada quando houver um segundo consumidor real.

Não existem ABI, userspace, scheduler, syscalls, filesystem ou drivers complexos na v0.0.3. A v0.0.3 adiciona um allocator de frames físicos de 4 KiB baseado no mapa do bootloader, sem alterar paginação ou criar heap.

## Propriedade e segurança

O modelo de segurança é owner-controlled: não há chave mestra do projeto, telemetria obrigatória ou autoridade remota necessária para operar a máquina. Secure Boot será opcional e, quando usado, deverá aceitar chaves controladas pelo proprietário.

A criptografia de dados e a criptografia ponta a ponta serão adicionadas em camadas posteriores. O kernel fornecerá isolamento, geração segura de aleatoriedade, proteção de memória e acesso controlado a chaves; protocolos de comunicação permanecerão em userspace.
