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

fn main() -> Result<(), slint::PlatformError> {
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

    liga!(on_gerar_chave, &["ssh", "gen"]);
    liga!(on_importar_chave, &["ssh", "import", "--paste"]);
    liga!(on_adicionar_host, &["vps", "add"]);
    liga!(on_destrancar_cofre, &["vault", "status"]);
    liga!(on_criar_cofre, &["vault", "init"]);

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
            w.set_msg(no_terminal(&["vps", "exec", &h.alias]).into());
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
        for proibido in ["LineEdit", "TextInput", "input-type", "password"] {
            assert!(
                !ui.contains(proibido),
                "`{proibido}` na UI: um campo de entrada aqui é onde uma passphrase acabaria"
            );
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
