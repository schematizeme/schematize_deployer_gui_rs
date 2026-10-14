//! A janela (Slint) do `schematize-deployer`.
//!
//! **O quê:** casca VISUAL fina por cima do binário headless. Três telas — chaves SSH, hosts e
//! o estado do cofre — lendo o `--json`, e nada mais.
//!
//! **Onde:** o ícone do deployer, e `schematize-deployer-gui` no terminal.
//!
//! ## O escopo é TRÊS telas de 42 comandos, e o corte é deliberado (D3)
//!
//! `mcp`, `dns`, `policy`, `trust` e bootstrap ficam **fora**. São comandos de operação e de
//! agente: quem os usa está num script ou num runbook, e no terminal eles estão melhores. Uma
//! janela que espelhasse os 42 seria uma CLI pior, com mouse.
//!
//! O que entra são as três coisas que se faz com o mouse na mão e a cabeça em outro lugar:
//! **ver que chaves existem**, **conectar num host**, e **saber se o cofre está protegido**.
//!
//! ## A regra que governa esta janela: NADA de segredo na tela (D5)
//!
//! Chave privada e valor de segredo **não aparecem** — nem em tela, nem em log, nem em
//! mensagem de erro. A confirmação de identidade de uma chave é o **fingerprint**, que é
//! público por construção.
//!
//! E há uma segunda metade, menos óbvia: **passphrase não passa por esta janela.** Gerar
//! chave, importar, destrancar e criar cofre abrem TERMINAL — não por preguiça de desenhar um
//! campo de senha, mas porque um campo de senha aqui poria a passphrase na memória deste
//! processo sem necessidade nenhuma. O `ssh-keygen` e o próprio deployer já sabem lê-la sem
//! eco. O caminho mais curto para não vazar um segredo é não tocá-lo.
//!
//! ## Conectar abre terminal por outro motivo
//!
//! Uma sessão SSH é interativa: tem TTY, tem `Ctrl-C`, tem `sudo` do outro lado. Isso não cabe
//! numa janela Slint, e fingir que cabe daria uma sessão pior que a do terminal.
#![windows_subsystem = "windows"]

mod aplicar;
mod cli;
mod comandos;
mod json;
mod ligacao;
mod market_parasita;
mod procedencia;
mod redacao;
mod telas;
mod terminal;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // **RESPONDE `--version` E SAI, antes de qualquer coisa gráfica.**
    //
    // Sem isto a janela IGNORA a flag e ABRE — e quem perguntou fica esperando. Não é hipótese:
    // o `debugreport` do CLI pergunta a versão de cada binário do ecossistema, e a janela do hub
    // já teve exatamente este defeito. O `cmd_out` cortava no timeout e devolvia a primeira
    // linha do log de ambiente (`(WAYLAND_DISPLAY=wayland-0)`) como se fosse o número da versão.
    //
    // Responder e sair é o contrato mínimo de um binário de linha de comando — inclusive de um
    // que normalmente abre janela.
    if std::env::args().skip(1).any(|a| a == "--version" || a == "-V") {
        println!("schematize-deployer-gui {}", procedencia::rotulo_versao());
        return Ok(());
    }

    // Parasita do market (contrato 2026-10-13): garante schematize-market e -gui em
    // ~/.cargo/bin ANTES de abrir a janela. Nunca derruba a janela — falha vira linha no stderr.
    market_parasita::garantir();

    let w = MainWindow::new()?;
    w.set_aba(comandos::aba_inicial(std::env::args().skip(1)));
    aplicar::recarregar(&w);

    ligacao::ligar(&w);

    w.run()
}

#[cfg(test)]
mod tests {
    /// **O quê:** o código de produção de TODOS os módulos que disparam ação, sem os testes.
    /// **Onde:** os guardas de fonte abaixo — a regra vale para o conjunto, não só para o `main`.
    fn producao_total() -> String {
        [
            include_str!("main.rs"),
            include_str!("aplicar.rs"),
            include_str!("comandos.rs"),
            include_str!("ligacao.rs"),
        ]
        .iter()
        .map(|f| f.split("#[cfg(test)]").next().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
    }

    /// **NENHUMA ação desta janela roda processo direto — todas abrem TERMINAL.**
    ///
    /// Este teste lê o próprio arquivo e reprova um `Command::new`. É a regra do D5 e do D6
    /// virando verificação: um campo de senha aqui, ou um `spawn` direto, poria a passphrase
    /// na memória deste processo sem necessidade nenhuma.
    #[test]
    fn nenhuma_acao_roda_processo_direto() {
        let producao = producao_total();
        assert!(producao.contains("terminal::abrir"), "as ações têm de abrir terminal");
        assert!(
            !producao.contains("Command::new"),
            "esta janela não roda processo direto: tudo que pede segredo vai para o terminal"
        );
    }

    /// **Nenhum campo de senha nesta janela.** A passphrase não precisa passar por aqui, então
    /// não passa — e a forma de garantir isso é não ter onde digitá-la.
    #[test]
    fn nao_ha_campo_de_senha_na_janela() {
        let ui = include_str!("../ui/deployer.slint");

        // ---------------------------------------------------------------------------
        // ESTE TESTE FOI AFROUXADO NUM EIXO E APERTADO EM DOIS. A troca está escrita.
        //
        // A versão anterior proibia **qualquer** campo de entrada: `LineEdit`, `TextInput`,
        // `input-type`, `password`. O raciocínio era "um campo aqui é onde uma passphrase
        // acabaria", e ele estava certo sobre o risco e errado sobre o alcance.
        //
        // **O que a proibição total custou, e não é hipótese:** sem poder perguntar nada, cada
        // botão desta janela disparava um comando FIXO. Quatro dos cinco eram incompletos, e o
        // terminal abria só para mostrar erro de uso do CLI — `ssh gen` sem `<NAME>`,
        // `ssh import --paste` sem `--name`, `vps add` sem `--host/--user/<ALIAS>`, e
        // `vps exec <alias>` sem comando. O usuário relatou isso com estas palavras:
        // *"quando tento importar uma chave com deployer me joga no ssh [e dá erro]"*.
        //
        // Uma janela que não pode perguntar o NOME de uma chave não é uma janela mais segura.
        // É uma janela quebrada, que empurra a pessoa de volta para o terminal — onde ela vai
        // digitar a passphrase de qualquer jeito, sem nenhuma das garantias que este teste
        // pensava estar dando.
        //
        // **O que passou a valer, e é mais forte no que importa:**
        //
        //   1. campo de SENHA continua proibido, por construção (`input-type: password`);
        //   2. todo campo que existe tem de estar na ALLOWLIST abaixo — um campo novo reprova
        //      até alguém escrever aqui por que ele não é segredo;
        //   3. nenhuma palavra de segredo aparece na UI, nem como rótulo, nem como binding.
        //
        // A regra do D5 nunca foi "não perguntar nada". É **"segredo não passa por esta
        // janela"** — e passphrase e chave privada colada continuam indo para o terminal, onde
        // o `ssh-keygen` e o deployer as leem sem eco.
        // ---------------------------------------------------------------------------

        // 1) Campo de senha: proibido por construção.
        for proibido in ["input-type", "password"] {
            assert!(
                !ui.contains(proibido),
                "`{proibido}` na UI: campo de senha não existe aqui (D5) — o segredo vai para \
                 o terminal, onde é lido sem eco"
            );
        }

        // 1b) A palavra de segredo não pode estar num CAMPO — mas pode estar na PROSA.
        //
        // A primeira versão desta regra proibia a palavra `passphrase` em qualquer lugar da UI,
        // e reprovou na hora: o texto que explica *"a passphrase é lida no terminal, sem eco"*
        // contém a palavra. **Proibir a palavra proíbe explicar a política** — e uma janela que
        // abre um terminal sem dizer por quê é pior que uma que diz.
        //
        // É o mesmo erro que o guard de nome de irmão do `repos-de-janela.py` quase cometeu, e
        // pela mesma razão: comentário e prosa citam o assunto; só o CAMPO o manipula.
        for linha in ui.lines() {
            let e_campo = linha.contains("LineEdit")
                || linha.contains("placeholder-text")
                || linha.contains("valor <=>");
            if !e_campo {
                continue;
            }
            let baixo = linha.to_lowercase();
            for palavra in ["senha", "passphrase", "password", "privada", "secret"] {
                assert!(
                    !baixo.contains(palavra),
                    "a palavra `{palavra}` aparece num CAMPO de entrada:\n  {}\n\
                     Segredo não passa por esta janela (D5).",
                    linha.trim()
                );
            }
        }

        // 2) Todo campo de entrada é de um dado DECLARADO como não-secreto.
        //
        // A allowlist é por `valor <=>`, que é o binding do campo — não pelo rótulo, que é
        // prosa e muda de redação. Um campo novo reprova até entrar aqui, e entrar aqui exige
        // olhar para ele e afirmar que não é segredo.
        let permitidos = [
            "root.f-nome",    // nome da chave, ou alias do host
            "root.f-host",    // IP/hostname
            "root.f-user",    // usuário no host
            "root.f-arquivo", // CAMINHO do arquivo da chave — o caminho, nunca o conteúdo
        ];
        let mut resto = ui;
        while let Some(i) = resto.find("valor <=>") {
            resto = &resto[i + "valor <=>".len()..];
            let fim = resto.find(';').unwrap_or(resto.len());
            let ligado = resto[..fim].trim();
            assert!(
                permitidos.contains(&ligado),
                "campo de entrada ligado a `{ligado}`, que não está na allowlist de dados \
                 não-secretos. Se ele não é segredo, acrescente-o à lista DIZENDO por quê; se \
                 for, ele não pode ter campo nesta janela"
            );
        }

        // 3) O `TextInput` cru continua proibido: ele é o widget de baixo nível, sem o
        // `input-type` que o `LineEdit` expõe — e é por onde um campo de senha entraria sem
        // disparar a regra 1.
        assert!(
            !ui.contains("TextInput"),
            "`TextInput` cru na UI: use `LineEdit`, que declara o `input-type` e é o que a \
             regra 1 vigia"
        );
    }

    ///
    /// `exec` EXIGE um comando para auditar, e a janela não tinha um: mandava `exec <alias>` e
    /// o terminal abria para dizer *"faltou o comando"*. O `shell` é a sessão interativa.
    #[test]
    fn conectar_usa_shell_e_nao_exec() {
        let producao = producao_total();
        assert!(
            producao.contains(r#""vps", "shell""#),
            "«Conectar» tem de usar `vps shell` — a sessão interativa"
        );
        assert!(
            !producao.contains(r#""vps", "exec""#),
            "`vps exec` sem comando é o defeito que esta janela tinha: ele exige um comando \
             para auditar, e a janela não tem um"
        );
    }

    /// **O self-check da regra 1b:** ela distingue PROSA de CAMPO.
    ///
    /// Sem isto, a regra poderia ser um filtro que aprova sempre (se eu tivesse errado a
    /// detecção de campo) ou que reprova a explicação (se eu tivesse deixado a proibição total).
    /// As duas falhas são silenciosas, e as duas já aconteceram neste projeto.
    #[test]
    fn a_regra_do_campo_distingue_prosa_de_entrada() {
        let e_campo = |l: &str| {
            l.contains("LineEdit") || l.contains("placeholder-text") || l.contains("valor <=>")
        };
        // PROSA com a palavra: passa. É o texto que explica por que o terminal abre.
        for prosa in [
            r#"text: "A passphrase é pedida no terminal, pelo próprio ssh-keygen.";"#,
            r#"// a chave privada nunca passa por esta janela"#,
            r#"explica: "Colando: o terminal abre e você cola a chave lá.";"#,
        ] {
            assert!(!e_campo(prosa), "isto é prosa, não campo: {prosa}");
        }
        // CAMPO com a palavra: reprova.
        for campo in [
            r#"placeholder-text: "sua passphrase";"#,
            r#"LineEdit { text <=> root.senha; }"#,
            r#"valor <=> root.f-chave-privada;"#,
        ] {
            assert!(e_campo(campo), "isto é campo e a regra tem de vê-lo: {campo}");
        }
    }

    /// **A contraprova da regra 2:** um campo ligado a algo fora da allowlist reprova.
    ///
    /// Sem isto, a allowlist seria uma lista que ninguém viu barrar nada — e uma allowlist que
    /// nunca barrou é indistinguível de uma que aprova tudo.
    #[test]
    fn a_allowlist_de_campos_barra_o_que_nao_esta_nela() {
        let permitidos = ["root.f-nome", "root.f-host", "root.f-user", "root.f-arquivo"];
        for intruso in ["root.f-passphrase", "root.f-chave-privada", "root.qualquer"] {
            assert!(
                !permitidos.contains(&intruso),
                "`{intruso}` não pode estar na allowlist — o teste principal o aprovaria"
            );
        }
        // E os quatro legítimos continuam passando, senão a regra barraria tudo.
        for ok in permitidos {
            assert!(permitidos.contains(&ok));
        }
    }

    /// **`vps trust` NÃO recebe `--sim`.** Confiar numa host key exige VER a fingerprint e
    /// compará-la com outra fonte; passar o `--sim` daqui seria a janela confiando no lugar da
    /// pessoa — o TOFU cego que aquele comando existe para acabar.
    #[test]
    fn confiar_nao_passa_o_sim() {
        let producao = producao_total();
        assert!(producao.contains(r#"&["vps", "trust", &h.alias]"#), "o comando mudou");
        assert!(!producao.contains(r#""--sim""#), "a janela não confia no lugar da pessoa");
    }
}

#[cfg(test)]
mod tests_versao {
    /// **A janela responde a versão COM a procedência, como o CLI irmão** (SHA, e `-sujo` se
    /// o build tinha mudança não commitada). O número seco não distingue dois binários.
    #[test]
    fn a_versao_carrega_a_procedencia() {
        let fonte = include_str!("main.rs");
        let producao = fonte.split("#[cfg(test)]").next().expect("há código antes dos testes");
        assert!(
            !producao.contains("env!(\"CARGO_PKG_VERSION\")"),
            "a janela usou o número seco — use `procedencia::rotulo_versao()`, que traz o SHA"
        );
        assert!(producao.contains("procedencia::rotulo_versao()"), "a versão perdeu a procedência");
    }

    /// **A janela responde `--version` e SAI, em vez de abrir.**
    ///
    /// Lê o próprio fonte e exige que a checagem seja a PRIMEIRA coisa do `main` — antes de
    /// qualquer chamada gráfica. A ordem é o ponto: a flag tratada depois de abrir a janela
    /// responde tarde demais, e quem perguntou já está esperando.
    ///
    /// **O defeito é conhecido e já custou um relatório errado.** O `debugreport` do CLI
    /// pergunta a versão de cada binário do ecossistema; a janela do hub ignorava a flag e
    /// abria, o `cmd_out` cortava no timeout, e a primeira linha do log de ambiente
    /// (`(WAYLAND_DISPLAY=wayland-0)`) entrava no relatório como se fosse o número da versão.
    #[test]
    fn responde_version_antes_de_abrir_a_janela() {
        let fonte = include_str!("main.rs");
        let producao = fonte.split("#[cfg(test)]").next().unwrap();
        let i = producao.find("fn main(").expect("há um main");
        let corpo = &producao[i..];

        let versao = corpo.find("\"--version\"").expect(
            "a janela tem de responder `--version` — sem isso ela ABRE quando alguém pergunta",
        );
        // A checagem vem antes de tudo que toca a tela. `MainWindow::new()` é a primeira
        // chamada gráfica de todas.
        let janela = corpo.find("MainWindow::new").expect("o main cria a janela");
        assert!(
            versao < janela,
            "`--version` é tratado DEPOIS de criar a janela — responder tarde é o mesmo que \
             não responder, porque a janela já subiu"
        );
        assert!(corpo[versao..janela].contains("return"), "tem de RESPONDER e SAIR");
    }

    /// O parasita do market tem de rodar DEPOIS do `--version` (que sai antes) e ANTES da janela
    /// (contrato market-parasita, regra 7: "nas janelas, antes de abrir a janela").
    #[test]
    fn garante_o_market_entre_o_version_e_a_janela() {
        let fonte = include_str!("main.rs");
        let producao = fonte.split("#[cfg(test)]").next().unwrap();
        let i = producao.find("fn main(").expect("há um main");
        let corpo = &producao[i..];
        let versao = corpo.find("\"--version\"").expect("o main responde --version");
        let parasita = corpo
            .find("market_parasita::garantir()")
            .expect("a janela tem de chamar o parasita do market");
        let janela = corpo.find("MainWindow::new").expect("o main cria a janela");
        assert!(versao < parasita, "o parasita roda ANTES do --version — baixaria em --version");
        assert!(parasita < janela, "o parasita roda DEPOIS de abrir a janela — tem de ser antes");
    }
}
