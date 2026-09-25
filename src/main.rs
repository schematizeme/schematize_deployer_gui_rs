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

mod cli;
mod json;
mod redacao;
mod telas;
mod terminal;

slint::include_modules!();

/// **O quê:** aplica as chaves na janela — inclusive quando são `Err`.
///
/// **Onde:** a abertura e o botão de recarregar.
///
/// **O `Err` NÃO vira lista vazia.** Lista vazia é a tela dizendo "você não tem chave nenhuma",
/// que é uma afirmação — e falsa, quando o que houve foi não ter conseguido perguntar.
fn aplicar_chaves(w: &MainWindow, r: Result<Vec<telas::Chave>, String>) {
    match r {
        Ok(v) => {
            w.set_chaves_erro(slint::SharedString::new());
            let modelo: Vec<ChaveUI> = v
                .iter()
                .map(|k| ChaveUI {
                    nome: k.nome.clone().into(),
                    algoritmo: k.algoritmo.clone().into(),
                    fingerprint: k.fingerprint.clone().into(),
                    comentario: k.comentario.clone().into(),
                    caminho_publica: k.caminho_publica.clone().into(),
                    op_rotulo: slint::SharedString::new(),
                })
                .collect();
            w.set_chaves(slint::ModelRc::new(slint::VecModel::from(modelo)));
        }
        Err(e) => {
            w.set_chaves_erro(e.into());
            w.set_chaves(slint::ModelRc::new(slint::VecModel::from(Vec::<ChaveUI>::new())));
        }
    }
}

/// **O quê:** aplica os hosts na janela — inclusive quando são `Err`.
/// **Onde:** a abertura e o recarregar.
fn aplicar_hosts(w: &MainWindow, r: Result<Vec<telas::Host>, String>) {
    match r {
        Ok(v) => {
            w.set_hosts_erro(slint::SharedString::new());
            let modelo: Vec<HostUI> = v
                .iter()
                .map(|h| HostUI {
                    alias: h.alias.clone().into(),
                    destino: h.destino.clone().into(),
                    chave: h.chave.clone().into(),
                    ambiente: h.ambiente.clone().into(),
                    producao: h.producao,
                    modo: h.modo.clone().into(),
                    confiado: h.confiado,
                    fingerprint: h.fingerprint.clone().into(),
                    fronteira: h.fronteira.clone().into(),
                    fronteira_servidor: h.fronteira_servidor,
                    op_rotulo: slint::SharedString::new(),
                })
                .collect();
            w.set_hosts(slint::ModelRc::new(slint::VecModel::from(modelo)));
        }
        Err(e) => {
            w.set_hosts_erro(e.into());
            w.set_hosts(slint::ModelRc::new(slint::VecModel::from(Vec::<HostUI>::new())));
        }
    }
}

/// **O quê:** aplica o estado do cofre na janela. **Onde:** a abertura e o recarregar.
///
/// **Cofre ausente NÃO passa pelo `Err`:** ele chega como `Ok` com `existe: false`, porque é o
/// estado inicial de toda instalação. O `Err` aqui é só "não consegui falar com o deployer".
fn aplicar_cofre(w: &MainWindow, r: Result<telas::Cofre, String>) {
    match r {
        Ok(c) => {
            w.set_cofre_erro(slint::SharedString::new());
            w.set_cofre_existe(c.existe);
            w.set_cofre_caminho(c.caminho.into());
            w.set_cofre_tamanho(c.tamanho.into());
            w.set_cofre_permissao(c.permissao.into());
            w.set_cofre_permissao_ok(c.permissao_ok);
            w.set_cofre_kdf(c.kdf.into());
            w.set_cofre_kdf_fraco(c.kdf_fraco);
        }
        Err(e) => {
            w.set_cofre_erro(e.into());
            // Sem leitura, a janela não afirma NADA sobre o cofre — nem que existe, nem que
            // não. `existe: false` com erro na tela mostraria o convite a criar um cofre que
            // talvez já exista.
            w.set_cofre_existe(false);
        }
    }
}

/// **O quê:** lê os três contratos e aplica na janela. **Onde:** a abertura e o recarregar.
fn recarregar(w: &MainWindow) {
    aplicar_chaves(w, cli::ler_json(&["ssh", "list"]).and_then(|t| telas::ler_chaves(&t)));
    aplicar_hosts(w, cli::ler_json(&["vps", "list"]).and_then(|t| telas::ler_hosts(&t)));
    aplicar_cofre(w, cli::ler_json(&["vault", "status"]).and_then(|t| telas::ler_cofre(&t)));
    // A versão sai de qualquer um dos três; o `ssh list` é o mais barato dos três.
    if let Ok(t) = cli::ler_json(&["ssh", "list"]) {
        if let Ok(j) = json::ler(&t) {
            w.set_versao(j.str_ou_vazio("deployer").into());
        }
    }
}

/// **O quê:** dispara um subcomando num TERMINAL e devolve o que dizer à pessoa.
///
/// **Onde:** todas as ações desta janela — sem exceção, e a ausência de exceção é o desenho.
///
/// **Por que TUDO passa pelo terminal aqui**, e não só o que muda a máquina como no optimizer:
///
/// - **gerar, importar, destrancar, criar cofre** — pedem passphrase. Um campo de senha nesta
///   janela poria o segredo na memória deste processo sem necessidade; o `ssh-keygen` e o
///   deployer já sabem lê-la sem eco. O caminho mais curto para não vazar é não tocar.
/// - **conectar** — é uma sessão interativa, com TTY e `Ctrl-C`. Não cabe numa janela Slint.
/// - **adicionar host, conferir host key** — pedem confirmação e mostram a fingerprint para a
///   pessoa comparar. O prompt já existe no CLI, e reimplementá-lo aqui seria duplicar a
///   decisão de segurança em dois lugares que podem divergir.
///
/// **Sem terminal a janela NÃO some com o problema:** devolve o comando para a pessoa rodar
/// onde quiser. Botão que não faz nada e não diz nada é bug do software (§37.48).
fn no_terminal(args: &[&str]) -> String {
    let bin = cli::bin().display().to_string();
    let cmd = cli::comando_para_terminal(&bin, args);
    if terminal::abrir(&cmd) {
        "terminal aberto. Ao terminar, clique em «Recarregar».".to_string()
    } else {
        format!("não achei um terminal. Rode: {bin} {}", args.join(" "))
    }
}

/// **O quê:** a aba em que a janela abre, lida da linha de comando.
///
/// **Onde:** [`main`]. `--hosts` e `--cofre` abrem nas respectivas; sem argumento, em Chaves.
///
/// **Por que Chaves é o padrão:** é o que existe antes de tudo o mais. Sem chave não há host
/// para conectar, e o cofre é opcional. Quem clica no ícone sem pedir nada em especial está,
/// na maior parte das vezes, atrás de uma chave.
///
/// **Argumento desconhecido não é erro.** Sair com erro por uma flag digitada errado trocaria
/// uma janela que funciona por nenhuma.
fn aba_inicial(args: impl Iterator<Item = String>) -> i32 {
    for a in args {
        match a.as_str() {
            // `--chaves` é redundante com o default e existe de propósito: o hub passa a aba
            // que a pessoa pediu, sempre, e um chamador que precisa saber que "sem flag é
            // chaves" é um chamador acoplado ao default. Se o default mudar um dia, quem
            // passou `--chaves` continua indo para chaves.
            "--chaves" | "--keys" => return 0,
            "--hosts" | "--vps" => return 1,
            "--cofre" | "--vault" => return 2,
            _ => {}
        }
    }
    0
}

/// **O quê:** o comando que o formulário aberto vai rodar, ou `None` se falta campo.
///
/// **Onde:** `on_form_confirmar`.
///
/// ## Esta função é o conserto
///
/// Antes, cada botão disparava um comando FIXO — e quatro dos cinco eram incompletos. O terminal
/// abria só para mostrar um erro de uso do CLI:
///
/// | botão | o que a janela mandava | o que o CLI respondia |
/// |---|---|---|
/// | Gerar chave | `ssh gen` | `required arguments were not provided: <NAME>` |
/// | Importar | `ssh import --paste` | `colando a chave eu não tenho de onde tirar o nome` |
/// | Adicionar host | `vps add` | `required: --host <HOST> --user <USER> <ALIAS>` |
/// | Conectar | `vps exec <alias>` | `faltou o comando` |
///
/// Abrir um terminal para exibir erro de argumento é pior que não ter botão: parece que o
/// software tentou e falhou, quando ele nem chegou a tentar.
///
/// **O que continua indo para o terminal:** a passphrase e a chave privada colada. Perguntá-las
/// aqui poria segredo na memória desta janela sem necessidade (D5) — o `ssh-keygen` e o deployer
/// já as leem sem eco.
fn comando_do_form(w: &MainWindow) -> Option<Vec<String>> {
    montar_comando(
        w.get_form(),
        w.get_f_nome().trim(),
        w.get_f_host().trim(),
        w.get_f_user().trim(),
        w.get_f_arquivo().trim(),
        w.get_f_colar(),
    )
}

/// **O quê:** a decisão de qual comando montar, como função PURA.
///
/// **Onde:** [`comando_do_form`], que só lê a janela; e os testes.
///
/// **Separada da janela de propósito:** construir uma `MainWindow` num teste exige um backend
/// gráfico, que não existe no CI. Com a decisão fora dela, as quatro formas de comando são
/// afirmáveis sem abrir janela nenhuma — e é a decisão que erra, não a leitura dos campos.
///
/// `None` significa "falta campo obrigatório". O botão já fica desabilitado nesse estado; isto
/// é a segunda barreira, para o caso de a primeira mudar.
fn montar_comando(
    form: i32,
    nome: &str,
    host: &str,
    user: &str,
    arquivo: &str,
    colar: bool,
) -> Option<Vec<String>> {
    match form {
        1 if !nome.is_empty() => Some(vec!["ssh".into(), "gen".into(), nome.into()]),
        2 if colar && !nome.is_empty() => Some(vec![
            "ssh".into(),
            "import".into(),
            "--paste".into(),
            "--name".into(),
            nome.into(),
        ]),
        2 if !colar && !arquivo.is_empty() => {
            let mut v: Vec<String> = vec!["ssh".into(), "import".into(), arquivo.into()];
            // O `--name` só entra quando foi dado: sem ele o CLI usa o nome do ARQUIVO, que é
            // o comportamento documentado — mandar `--name ""` o quebraria.
            if !nome.is_empty() {
                v.push("--name".into());
                v.push(nome.into());
            }
            Some(v)
        }
        3 if !nome.is_empty() && !host.is_empty() && !user.is_empty() => Some(vec![
            "vps".into(),
            "add".into(),
            nome.into(),
            "--host".into(),
            host.into(),
            "--user".into(),
            user.into(),
        ]),
        _ => None,
    }
}

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
        println!("schematize-deployer-gui {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let w = MainWindow::new()?;
    w.set_aba(aba_inicial(std::env::args().skip(1)));
    recarregar(&w);

    macro_rules! liga {
        ($cb:ident, $args:expr) => {{
            let weak = w.as_weak();
            w.$cb(move || {
                if let Some(w) = weak.upgrade() {
                    w.set_msg(no_terminal($args).into());
                }
            });
        }};
    }

    // Os dois que NÃO precisam de campo nenhum seguem disparando direto.
    liga!(on_destrancar_cofre, &["vault", "status"]);
    liga!(on_criar_cofre, &["vault", "init"]);

    // Os três que PRECISAM passam pelo formulário — ver `comando_do_form`.
    {
        let weak = w.as_weak();
        w.on_form_abrir(move |qual| {
            if let Some(w) = weak.upgrade() {
                // Limpa os campos ao ABRIR, e não ao fechar: fechar e reabrir o mesmo
                // formulário com o que foi digitado antes parece que o software guardou algo
                // que ele não guardou — e o valor velho vira um comando que ninguém revisou.
                w.set_f_nome(String::new().into());
                w.set_f_host(String::new().into());
                w.set_f_user(String::new().into());
                w.set_f_arquivo(String::new().into());
                w.set_f_colar(false);
                w.set_msg(String::new().into());
                w.set_form(qual);
            }
        });
    }
    {
        let weak = w.as_weak();
        w.on_form_fechar(move || {
            if let Some(w) = weak.upgrade() {
                w.set_form(0);
            }
        });
    }
    {
        let weak = w.as_weak();
        w.on_form_confirmar(move || {
            let Some(w) = weak.upgrade() else { return };
            match comando_do_form(&w) {
                Some(args) => {
                    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    w.set_msg(no_terminal(&refs).into());
                    w.set_form(0);
                }
                // Não deveria acontecer — o botão fica desabilitado enquanto falta campo. Mas
                // se acontecer, a janela DIZ o que falta em vez de abrir um terminal mudo.
                None => w.set_msg(
                    "faltou preencher um campo obrigatório (marcado com *).".to_string().into(),
                ),
            }
        });
    }

    {
        let weak = w.as_weak();
        w.on_recarregar(move || {
            if let Some(w) = weak.upgrade() {
                recarregar(&w);
                w.set_msg("estado relido.".into());
            }
        });
    }
    {
        let weak = w.as_weak();
        w.on_alternar_tema(move || {
            if let Some(w) = weak.upgrade() {
                w.set_dark(!w.get_dark());
            }
        });
    }
    // As ações POR LINHA precisam do alias/nome da linha clicada. `row_data` devolve `None`
    // para índice fora da lista — possível se a tela foi redesenhada entre o clique e este
    // callback —, e uma janela que morre no clique é pior que um clique que não faz nada.
    {
        let weak = w.as_weak();
        w.on_copiar_publica(move |i| {
            let Some(w) = weak.upgrade() else { return };
            use slint::Model;
            let Some(k) = w.get_chaves().row_data(i as usize) else { return };
            w.set_msg(no_terminal(&["ssh", "export", &k.nome, "--copy"]).into());
        });
    }
    {
        let weak = w.as_weak();
        w.on_conectar(move |i| {
            let Some(w) = weak.upgrade() else { return };
            use slint::Model;
            let Some(h) = w.get_hosts().row_data(i as usize) else { return };
            // `vps shell`, e não `vps exec`: `exec` EXIGE um comando para auditar, e a
            // janela não tinha um — mandava `exec <alias>` e o terminal abria para dizer
            // "faltou o comando". O `shell` é a sessão interativa, que é o que «Conectar» quer.
            w.set_msg(no_terminal(&["vps", "shell", &h.alias]).into());
        });
    }
    {
        let weak = w.as_weak();
        w.on_confiar(move |i| {
            let Some(w) = weak.upgrade() else { return };
            use slint::Model;
            let Some(h) = w.get_hosts().row_data(i as usize) else { return };
            // SEM `--sim`: quem confia numa host key tem de VER a fingerprint e comparar com
            // outra fonte. Passar o `--sim` daqui seria a janela confiando no lugar da pessoa,
            // que é exatamente o TOFU cego que o comando existe para acabar.
            w.set_msg(no_terminal(&["vps", "trust", &h.alias]).into());
        });
    }

    w.run()
}

#[cfg(test)]
mod tests_aba {
    use super::aba_inicial;

    /// Sem argumento, abre em Chaves — é o que existe antes de tudo o mais: sem chave não há
    /// host para conectar, e o cofre é opcional.
    #[test]
    fn sem_argumento_abre_em_chaves() {
        assert_eq!(aba_inicial(std::iter::empty()), 0);
    }

    /// **`--chaves` é EXPLÍCITO, e não redundante à toa.** Ele leva à mesma aba que a
    /// ausência de flag — mas um chamador que dependesse de "sem flag é chaves" ficaria
    /// acoplado ao default. O hub passa a aba que a pessoa pediu, sempre; se o default mudar
    /// um dia, quem passou `--chaves` continua indo para chaves.
    #[test]
    fn chaves_e_explicito_mesmo_sendo_o_default() {
        for flag in ["--chaves", "--keys"] {
            assert_eq!(aba_inicial([flag.to_string()].into_iter()), 0, "{flag}");
        }
    }

    /// Cada aba tem duas grafias: a do produto e a do subcomando que a alimenta.
    #[test]
    fn cada_aba_tem_a_sua_flag() {
        for flag in ["--hosts", "--vps"] {
            assert_eq!(aba_inicial([flag.to_string()].into_iter()), 1, "{flag}");
        }
        for flag in ["--cofre", "--vault"] {
            assert_eq!(aba_inicial([flag.to_string()].into_iter()), 2, "{flag}");
        }
    }

    /// **Argumento desconhecido NÃO derruba a janela.**
    #[test]
    fn argumento_desconhecido_nao_derruba_nada() {
        assert_eq!(aba_inicial(["--nao-existe".to_string()].into_iter()), 0);
        assert_eq!(aba_inicial(["".to_string(), "-".to_string()].into_iter()), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::montar_comando;

    /// **NENHUMA ação desta janela roda processo direto — todas abrem TERMINAL.**
    ///
    /// Este teste lê o próprio arquivo e reprova um `Command::new`. É a regra do D5 e do D6
    /// virando verificação: um campo de senha aqui, ou um `spawn` direto, poria a passphrase
    /// na memória deste processo sem necessidade nenhuma.
    #[test]
    fn nenhuma_acao_roda_processo_direto() {
        let fonte = include_str!("main.rs");
        let producao = fonte.split("#[cfg(test)]").next().unwrap();
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

    /// **Os comandos que os formulários montam — o conserto, afirmado.**
    ///
    /// Cada caso aqui é um dos comandos que ANTES saía incompleto e abria um terminal só para
    /// mostrar erro de uso do CLI.
    #[test]
    fn o_formulario_monta_o_comando_completo() {
        assert_eq!(
            montar_comando(1, "producao", "", "", "", false).unwrap(),
            ["ssh", "gen", "producao"],
            "`ssh gen` SEM nome era o erro: `required arguments were not provided: <NAME>`"
        );
        assert_eq!(
            montar_comando(2, "producao", "", "", "", true).unwrap(),
            ["ssh", "import", "--paste", "--name", "producao"],
            "colando, o `--name` é obrigatório — é EXATAMENTE o erro que o usuário viu"
        );
        assert_eq!(
            montar_comando(2, "", "", "", "/bk/id_ed25519", false).unwrap(),
            ["ssh", "import", "/bk/id_ed25519"],
            "de arquivo SEM nome: o CLI usa o nome do arquivo, e `--name \"\"` o quebraria"
        );
        assert_eq!(
            montar_comando(2, "outro", "", "", "/bk/id_ed25519", false).unwrap(),
            ["ssh", "import", "/bk/id_ed25519", "--name", "outro"],
            "de arquivo COM nome: o nome dado ganha"
        );
        assert_eq!(
            montar_comando(3, "srv", "10.0.0.1", "deploy", "", false).unwrap(),
            ["vps", "add", "srv", "--host", "10.0.0.1", "--user", "deploy"],
            "`vps add` sozinho não tem alias, host nem user"
        );
    }

    /// **Falta campo obrigatório → `None`, e nunca um comando pela metade.**
    ///
    /// É a regra inteira desta tela: um comando incompleto abre um terminal que só sabe dizer
    /// que o comando está incompleto.
    #[test]
    fn campo_obrigatorio_faltando_nao_monta_comando() {
        // Gerar sem nome.
        assert!(montar_comando(1, "", "", "", "", false).is_none());
        // Colar sem nome — o caso relatado.
        assert!(montar_comando(2, "", "", "", "", true).is_none());
        // Colar COM arquivo mas sem nome: o arquivo não salva o caso, porque `--paste` ignora
        // arquivo e o CLI continua sem de onde tirar o nome.
        assert!(montar_comando(2, "", "", "", "/bk/id", true).is_none());
        // Arquivo vazio no modo arquivo.
        assert!(montar_comando(2, "nome", "", "", "", false).is_none());
        // Host com qualquer um dos três em branco.
        assert!(montar_comando(3, "", "10.0.0.1", "deploy", "", false).is_none());
        assert!(montar_comando(3, "srv", "", "deploy", "", false).is_none());
        assert!(montar_comando(3, "srv", "10.0.0.1", "", "", false).is_none());
        // Formulário fechado não monta nada.
        assert!(montar_comando(0, "srv", "10.0.0.1", "deploy", "/bk/id", true).is_none());
    }

    /// **«Conectar» usa `vps shell`, e não `vps exec`.**
    ///
    /// `exec` EXIGE um comando para auditar, e a janela não tinha um: mandava `exec <alias>` e
    /// o terminal abria para dizer *"faltou o comando"*. O `shell` é a sessão interativa.
    #[test]
    fn conectar_usa_shell_e_nao_exec() {
        let producao = include_str!("main.rs").split("#[cfg(test)]").next().unwrap();
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
        let fonte = include_str!("main.rs");
        let producao = fonte.split("#[cfg(test)]").next().unwrap();
        assert!(producao.contains(r#"&["vps", "trust", &h.alias]"#), "o comando mudou");
        assert!(!producao.contains(r#""--sim""#), "a janela não confia no lugar da pessoa");
    }
}

#[cfg(test)]
mod tests_versao {
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
}
