//! Comandos disparados pela janela: terminal, aba inicial e montagem do comando do formulário.
//!
//! **O quê:** tudo que decide QUAL comando o deployer roda e COMO ele chega ao terminal.
//!
//! **Onde:** os callbacks ligados em `ligacao` e o `main` (aba inicial).

use crate::{cli, terminal, MainWindow};

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
pub fn no_terminal(args: &[&str]) -> String {
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
pub fn aba_inicial(args: impl Iterator<Item = String>) -> i32 {
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
pub fn comando_do_form(w: &MainWindow) -> Option<Vec<String>> {
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
pub(crate) fn montar_comando(
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
mod tests_form {
    use super::montar_comando;

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
}
