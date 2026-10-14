//! Ligação dos callbacks da janela (wiring).
//!
//! **O quê:** conecta cada callback do Slint à ação correspondente (terminal, formulário, recarregar).
//!
//! **Onde:** chamada uma vez pelo `main`, depois de criar a janela.

use crate::aplicar::recarregar;
use crate::comandos::{comando_do_form, no_terminal};
use crate::MainWindow;
use slint::ComponentHandle;

/// **O quê:** registra todos os callbacks da janela. **Onde:** `main`, antes de `w.run()`.
pub fn ligar(w: &MainWindow) {
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
}
