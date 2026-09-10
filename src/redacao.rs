//! REDAÇÃO — a segunda barreira, do lado da janela.
//!
//! **O quê:** apaga material de chave privada e segredo em claro de qualquer texto que a tela
//! vá exibir.
//!
//! **Onde:** [`crate::telas`], em todo campo de TEXTO LIVRE — comentário de chave, alias, nome
//! de host, caminho.
//!
//! ## Por que isto existe se o deployer já redige
//!
//! Porque **as duas metades nunca se atualizam no mesmo instante**. O `--json` do deployer
//! passa os campos de texto livre pela redação dele; esta janela é outro binário, distribuída
//! separadamente, e vai rodar contra deployers mais velhos que ela. Uma janela que só é segura
//! quando o outro lado está atualizado não é segura.
//!
//! A lição é fresca: a janela do market lia `status_text`, o market publicado ainda emitia
//! `status`, e a tela mostrou uma pílula vazia. Ali o custo foi um oval sem texto. Aqui o custo
//! seria uma chave privada na tela.
//!
//! ## O que ela NÃO tenta ser
//!
//! Não é a redação do deployer, que tem corpus de log legítimo e trata URL de conexão, flag de
//! senha e mais uma dúzia de casos. Aqui a entrada é **campo de exibição de um contrato**, não
//! log arbitrário: comentário de chave, alias, caminho. O conjunto do que pode aparecer é
//! pequeno, e a regra é a mais grosseira que serve — **na dúvida, apaga**, porque o custo de
//! apagar um comentário legítimo é a pessoa não ler um comentário, e o custo de deixar passar
//! é uma chave privada na tela.

/// O que substitui qualquer segredo detectado.
pub const REDIGIDO: &str = "<REDIGIDO>";

/// **O quê:** `true` se a palavra parece um token de serviço.
///
/// **Onde:** [`limpar`]. Prefixo conhecido seguido de corpo suficiente — sem o corpo, `sk-` é
/// só um pedaço de "sk-" numa frase, e apagar isso seria comer texto legítimo.
fn parece_token(p: &str) -> bool {
    for pfx in [
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "xoxb-",
        "xoxp-",
        "xapp-",
        "sk-",
        "re_",
        "AKIA",
        "ASIA",
    ] {
        if let Some(i) = p.find(pfx) {
            let corpo = p[i + pfx.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .count();
            if corpo >= 8 {
                return true;
            }
        }
    }
    // JWT: três partes base64url começando por `eyJ`.
    if let Some(i) = p.find("eyJ") {
        let corrida: String = p[i..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            .collect();
        let partes: Vec<&str> = corrida.split('.').collect();
        if partes.len() >= 3 && partes.iter().take(3).all(|x| !x.is_empty()) {
            return true;
        }
    }
    false
}

/// **O quê:** devolve o texto sem material de chave privada nem token.
///
/// **Onde:** todo campo de texto livre de [`crate::telas`].
///
/// **Um bloco PEM apaga o texto INTEIRO**, e não só as linhas do bloco. É a diferença entre
/// esta função e a redação do deployer, e é deliberada: lá a entrada é um log, onde o que está
/// em volta do bloco é diagnóstico útil que se quer preservar; aqui é um campo de exibição, e
/// um comentário de chave que contém um bloco PEM não tem nada de aproveitável em volta —
/// mostrar metade dele só daria à pessoa a impressão de que a tela está mostrando a chave.
///
/// **Nunca devolve vazio quando apagou algo:** devolve o marcador. Vazio pareceria campo que
/// não carregou, e a pessoa iria procurar o problema errado.
pub fn limpar(texto: &str) -> String {
    if texto.contains("PRIVATE KEY-----") || texto.contains("BEGIN OPENSSH") {
        return REDIGIDO.to_string();
    }
    let mut mudou = false;
    let limpo: Vec<String> = texto
        .split_whitespace()
        .map(|p| {
            if parece_token(p) {
                mudou = true;
                REDIGIDO.to_string()
            } else {
                p.to_string()
            }
        })
        .collect();
    if mudou {
        limpo.join(" ")
    } else {
        // Nada suspeito: devolve o ORIGINAL, com o espaçamento intacto. O `split_whitespace`
        // normalizaria espaços múltiplos, e reescrever em silêncio o que não era para mexer é
        // o oposto do que uma barreira deve fazer.
        texto.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O caso que motivou este módulo.** Um bloco PEM colado no comentário de uma chave.
    #[test]
    fn bloco_pem_apaga_o_texto_inteiro() {
        for entrada in [
            "-----BEGIN OPENSSH PRIVATE KEY-----\nb3Blbn\n-----END OPENSSH PRIVATE KEY-----",
            "olha minha chave: -----BEGIN RSA PRIVATE KEY----- MIIEow",
            "-----BEGIN EC PRIVATE KEY-----",
        ] {
            let s = limpar(entrada);
            assert_eq!(s, REDIGIDO, "o bloco tem de apagar o campo inteiro: {s}");
            assert!(!s.contains("BEGIN"));
        }
    }

    /// Tokens de serviço somem, e o resto da frase fica — aqui não há bloco a apagar inteiro.
    #[test]
    fn token_some_e_o_resto_fica() {
        let s = limpar("chave de deploy ghp_AbCdEf0123456789AbCdEf do robô");
        assert!(!s.contains("ghp_AbCdEf"), "{s}");
        assert!(s.contains("chave de deploy") && s.contains("do robô"), "{s}");
        assert!(s.contains(REDIGIDO), "{s}");
    }

    /// **O que NÃO pode ser tocado**, e é a metade que faz esta barreira ser usável.
    ///
    /// Ampliar heurística de redação sem provar ausência de falso-positivo troca um problema
    /// (segredo na tela) por outro (tela ilegível). O fingerprint é o caso crítico: ele é
    /// PÚBLICO, é a confirmação de identidade que o D5 manda usar, e apagá-lo tornaria a tela
    /// de chaves inútil.
    #[test]
    fn o_que_e_publico_passa_intacto() {
        for bom in [
            "SHA256:c4Z/1C1D9jm/7Am0aiGQEpp/Cb/hPzlRHIxTWr6n6dU",
            "schematize:luna@localhost.localdomain",
            "/home/u/.ssh/github.pub",
            "deploy@10.0.0.5:2222",
            "srv-01",
            // Palavras que CONTÊM prefixo de token mas não são token.
            "skeleton",
            "sk-",
            "re_",
            "resource_id",
            "AKIA",
            "ASIAtico",
            // Espaçamento preservado: reescrever o que não era para mexer é o oposto do que
            // uma barreira deve fazer.
            "dois  espaços   e\ttab",
            "",
        ] {
            assert_eq!(limpar(bom), bom, "falso-positivo: {bom:?}");
        }
    }

    /// **Nunca devolve vazio quando apagou algo.** Vazio pareceria campo que não carregou, e a
    /// pessoa iria procurar o problema errado.
    #[test]
    fn apagar_nunca_deixa_o_campo_em_branco() {
        assert!(!limpar("-----BEGIN OPENSSH PRIVATE KEY-----").is_empty());
        assert!(!limpar("ghp_AbCdEf0123456789AbCdEf").is_empty());
    }

    /// Idempotente: redigir o já-redigido não muda mais nada.
    #[test]
    fn e_idempotente() {
        for e in ["-----BEGIN OPENSSH PRIVATE KEY-----", "tok ghp_AbCdEf0123456789", "normal"] {
            let uma = limpar(e);
            assert_eq!(limpar(&uma), uma, "não é idempotente: {e}");
        }
    }
}
