//! Aplicação do estado lido do deployer na janela.
//!
//! **O quê:** converte os contratos `--json` (chaves, hosts, cofre) em modelos Slint.
//!
//! **Onde:** a abertura da janela e o botão «Recarregar» (ver `ligacao`).

use crate::{cli, json, telas};
use crate::{ChaveUI, HostUI, MainWindow};

/// **O quê:** aplica as chaves na janela — inclusive quando são `Err`.
///
/// **Onde:** a abertura e o botão de recarregar.
///
/// **O `Err` NÃO vira lista vazia.** Lista vazia é a tela dizendo "você não tem chave nenhuma",
/// que é uma afirmação — e falsa, quando o que houve foi não ter conseguido perguntar.
pub fn aplicar_chaves(w: &MainWindow, r: Result<Vec<telas::Chave>, String>) {
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
pub fn aplicar_hosts(w: &MainWindow, r: Result<Vec<telas::Host>, String>) {
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
pub fn aplicar_cofre(w: &MainWindow, r: Result<telas::Cofre, String>) {
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
pub fn recarregar(w: &MainWindow) {
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
