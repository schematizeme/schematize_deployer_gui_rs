# schematize_deployer_gui_rs

A janela (Slint) do `schematize-deployer`.

## O que ela é

Uma casca **visual fina** por cima do binário headless. Três telas — chaves SSH, hosts e o
estado do cofre —, lendo `ssh list --json`, `vps list --json` e `vault status --json`.

## Três telas de 42 comandos, e o corte é deliberado

`mcp`, `dns`, `policy`, `trust` e bootstrap ficam **fora**. São comandos de operação e de
agente: quem os usa está num script ou num runbook, e no terminal eles estão melhores. Uma
janela que espelhasse os 42 seria uma CLI pior, com mouse.

O que entra são as três coisas que se faz com o mouse na mão e a cabeça em outro lugar: **ver
que chaves existem**, **conectar num host**, e **saber se o cofre está protegido**.

## A regra que governa esta janela: nada de segredo na tela

Chave privada e valor de segredo **não aparecem** — nem em tela, nem em log, nem em mensagem
de erro. A confirmação de identidade de uma chave é o **fingerprint**, que é público por
construção.

E há uma segunda metade, menos óbvia: **passphrase não passa por esta janela.** Gerar chave,
importar, destrancar e criar cofre abrem TERMINAL — não por preguiça de desenhar um campo de
senha, mas porque um campo de senha aqui poria a passphrase na memória deste processo sem
necessidade nenhuma. O `ssh-keygen` e o próprio deployer já sabem lê-la sem eco.

Há três testes que travam isso:

- `nenhuma_acao_roda_processo_direto` — lê o `main.rs` e reprova um `Command::new`.
- `nao_ha_campo_de_senha_na_janela` — lê o `.slint` e reprova `LineEdit`/`TextInput`.
- `nenhum_modelo_carrega_segredo` — varre os três modelos atrás de bloco PEM e token.

O terceiro **reprovou de verdade**, num caso que os tipos não cobrem: um bloco PEM colado no
comentário de uma chave. Daí `redacao.rs`, a segunda barreira do lado da janela — porque esta
janela é outro binário, e vai rodar contra deployers mais velhos que ela.

## Conectar abre terminal por outro motivo

Uma sessão SSH é interativa: tem TTY, tem `Ctrl-C`, tem `sudo` do outro lado. Isso não cabe
numa janela Slint, e fingir que cabe daria uma sessão pior que a do terminal.

## Rodar

```sh
cargo run --release            # abre em Chaves
cargo run --release -- --hosts # abre em Hosts
cargo run --release -- --cofre # abre em Cofre
```
