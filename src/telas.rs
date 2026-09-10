//! O DOMÍNIO das três telas — o `--json` do deployer virando o que a janela desenha.
//!
//! **O quê:** lê `ssh list --json`, `vps list --json` e `vault status --json`, e produz o
//! modelo das telas de chaves, hosts e cofre.
//!
//! **Onde:** a janela. Tudo aqui é PURO — entra texto, sai modelo —, e é isso que permite
//! testar as três telas sem o deployer instalado na máquina de quem roda a suíte.
//!
//! ## A regra que este arquivo existe para cumprir: NADA de segredo na tela (D5)
//!
//! Este é o único dos três apps que guarda segredo. **Chave privada e valor de segredo não
//! aparecem** — nem em tela, nem em log, nem em mensagem de erro. A confirmação de identidade
//! de uma chave é o **fingerprint**, que é público por construção: ele é o hash da chave
//! pública, e serve exatamente para confirmar sem mostrar.
//!
//! Aqui isso é estrutural antes de ser disciplina: o contrato do deployer **não traz** material
//! privado, e os tipos deste arquivo não têm campo onde ele caberia. Por cima disso há um teste
//! que varre cada modelo montado atrás de marcas de segredo e reprova se achar — e esse teste
//! foi visto reprovando antes de valer.
//!
//! **E ele reprovou de verdade**, num caso que os tipos não cobrem: um bloco PEM colado no
//! COMENTÁRIO de uma chave. Comentário é texto livre digitado por gente, e nenhum tipo impede
//! isso. Por isso todo campo de texto livre daqui passa por [`crate::redacao::limpar`] antes de
//! virar modelo — a segunda barreira, do lado da janela, para o dia em que ela rodar contra um
//! deployer mais velho que ela.
//!
//! **A tela do cofre mostra QUE ele existe, não o que há dentro.** Onde o arquivo está, que
//! permissão tem, com que custo de KDF foi criado. As chaves guardadas só aparecem depois de
//! destrancar, e mesmo então só os NOMES.
//!
//! ## Cofre trancado é estado de primeira classe, não erro
//!
//! Quem abre a tela do cofre numa máquina sem cofre não errou nada: é o estado inicial de toda
//! instalação. Uma tela de erro ali seria lida como "quebrou", e mandaria a pessoa procurar
//! problema onde não há.

use crate::json::{self, Json};
use crate::redacao::limpar;

// ---------------------------------------------------------------------------
// Tela 1 — CHAVES
// ---------------------------------------------------------------------------

/// Uma chave SSH, como a tela a mostra.
///
/// **Não há campo para a privada, e a ausência é a barreira.** O contrato não a traz e este
/// tipo não teria onde guardá-la — acrescentar um campo assim é a mudança que o teste de
/// varredura reprova.
#[derive(Debug, Clone, PartialEq)]
pub struct Chave {
    pub nome: String,
    pub algoritmo: String,
    /// O hash da chave PÚBLICA. É a confirmação de identidade que o D5 manda usar.
    pub fingerprint: String,
    pub comentario: String,
    /// Caminho da PÚBLICA. O da privada não entra: não é material de chave, mas apontar para
    /// ela num modelo de tela é o primeiro passo para alguém a ler.
    pub caminho_publica: String,
}

/// **O quê:** as chaves, a partir do `ssh list --json`. **Onde:** a tela de chaves. PURA.
///
/// **Lista vazia é FATO, e `Err` é outra coisa.** Quem não tem chave nenhuma precisa ver
/// "nenhuma chave ainda, gere a primeira"; quem não conseguiu falar com o deployer precisa ver
/// o motivo. Achatar os dois faria a tela afirmar o que não sabe.
pub fn ler_chaves(texto: &str) -> Result<Vec<Chave>, String> {
    let j = json::ler(texto).map_err(|e| format!("resposta do deployer ilegível: {e}"))?;
    Ok(j.arr("keys")
        .iter()
        .map(|o| Chave {
            nome: limpar(&o.str_ou_vazio("name")),
            algoritmo: o.str_ou_vazio("kind"),
            fingerprint: o.str_ou_vazio("fingerprint"),
            comentario: limpar(&o.str_ou_vazio("comment")),
            caminho_publica: limpar(&o.str_ou_vazio("public_path")),
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Tela 2 — HOSTS
// ---------------------------------------------------------------------------

/// Um host registrado, como a tela o mostra.
#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub alias: String,
    /// `usuario@host:porta`, montado aqui porque é como a pessoa pensa no destino.
    pub destino: String,
    /// O NOME da chave gerenciada — nunca o conteúdo dela.
    pub chave: String,
    /// `dev` | `hml` | `prd`, já em texto de tela.
    pub ambiente: String,
    /// O ambiente é `prd`? É o que a tela usa para destacar — errar em produção custa mais.
    pub producao: bool,
    /// Modo da política, em texto de tela.
    pub modo: String,
    /// A host key está pinada?
    pub confiado: bool,
    /// O fingerprint da host key — público, e é o que a pessoa confere.
    pub fingerprint: String,
    /// O que este host garante HOJE, em palavras.
    pub fronteira: String,
    /// Tem fronteira do lado do servidor? Sem ela, só a política do cliente protege.
    pub fronteira_servidor: bool,
}

/// **O quê:** o ambiente em texto de tela, a partir do slug estável do contrato.
/// **Onde:** [`ler_hosts`].
fn ambiente(slug: &str) -> String {
    match slug {
        "dev" => "desenvolvimento".into(),
        "hml" => "homologação".into(),
        "prd" => "produção".into(),
        outro => outro.to_string(),
    }
}

/// **O quê:** o modo da política em texto de tela. **Onde:** [`ler_hosts`].
fn modo(slug: &str) -> String {
    match slug {
        "readonly" => "só leitura".into(),
        "opsverbs" => "verbos de operação".into(),
        "livre" => "livre".into(),
        outro => outro.to_string(),
    }
}

/// **O quê:** o que o nível de fronteira GARANTE, em palavras. **Onde:** [`ler_hosts`].
///
/// **A frase do `sem` é a mais importante das três**, e é a única que a pessoa precisa ler
/// antes de confiar. Um host sem fronteira do lado do servidor está protegido só pela política
/// que roda nesta máquina — o que pega acidente, e não intenção. Escrever "sem" e deixar a
/// pessoa adivinhar o que isso significa seria informação que não informa.
fn fronteira(slug: &str) -> String {
    match slug {
        "root" => "o sshd recusa o que estiver fora do catálogo, e o shim é do root".into(),
        "usuario" => {
            "o sshd recusa o que estiver fora do catálogo; o shim é do próprio usuário".into()
        }
        "sem" => "SEM fronteira no servidor — só a política deste cliente protege".into(),
        outro => outro.to_string(),
    }
}

/// **O quê:** os hosts, a partir do `vps list --json`. **Onde:** a tela de hosts. PURA.
pub fn ler_hosts(texto: &str) -> Result<Vec<Host>, String> {
    let j = json::ler(texto).map_err(|e| format!("resposta do deployer ilegível: {e}"))?;
    Ok(j.arr("hosts")
        .iter()
        .map(|o| {
            let porta = match o.get("port") {
                Some(Json::Num(n)) => format!("{n:.0}"),
                _ => String::new(),
            };
            let amb = o.str_ou_vazio("env");
            Host {
                alias: limpar(&o.str_ou_vazio("alias")),
                destino: format!("{}@{}:{porta}", o.str_ou_vazio("user"), o.str_ou_vazio("host")),
                chave: limpar(&o.str_ou_vazio("key_name")),
                producao: amb == "prd",
                ambiente: ambiente(&amb),
                modo: modo(&o.str_ou_vazio("mode")),
                confiado: o.bool("host_key_pinned").unwrap_or(false),
                fingerprint: o.str_ou_vazio("fingerprint"),
                fronteira: fronteira(&o.str_ou_vazio("boundary")),
                fronteira_servidor: o.bool("server_side_boundary").unwrap_or(false),
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Tela 3 — COFRE
// ---------------------------------------------------------------------------

/// O estado do cofre, como a tela o mostra. **Nada aqui vem de DENTRO dele.**
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cofre {
    /// O arquivo existe? `false` é o estado inicial de toda instalação, não um erro.
    pub existe: bool,
    pub caminho: String,
    pub tamanho: String,
    /// A permissão octal, em texto.
    pub permissao: String,
    /// A permissão é `600`? Qualquer outra deixa o arquivo legível por mais gente do que devia.
    pub permissao_ok: bool,
    /// Os custos do Argon2id, em texto — lidos do cabeçalho, que viaja em claro de propósito.
    pub kdf: String,
    /// O custo de memória está abaixo do padrão atual do app?
    pub kdf_fraco: bool,
}

/// **O quê:** o estado do cofre, a partir do `vault status --json`. **Onde:** a tela. PURA.
///
/// **Cofre ausente devolve `Ok`, não `Err`.** Quem abre a tela numa máquina sem cofre não
/// errou nada: é o estado inicial de toda instalação. Uma tela de erro ali seria lida como
/// "quebrou", e mandaria a pessoa procurar problema onde não há.
pub fn ler_cofre(texto: &str) -> Result<Cofre, String> {
    let j = json::ler(texto).map_err(|e| format!("resposta do deployer ilegível: {e}"))?;
    let existe = j.bool("exists").unwrap_or(false);
    let kdf = match j.get("kdf") {
        Some(o @ Json::Obj(_)) => {
            let n = |k: &str| match o.get(k) {
                Some(Json::Num(v)) => format!("{v:.0}"),
                _ => "?".to_string(),
            };
            // Em MiB porque é como o Argon2 documenta o custo, e KiB dá um número de seis
            // dígitos que ninguém compara de cabeça.
            let mib = match o.get("m_cost") {
                Some(Json::Num(v)) => format!("{:.0} MiB", v / 1024.0),
                _ => "? MiB".to_string(),
            };
            format!("memória {mib} · {} passagens · {} vias", n("t_cost"), n("p_cost"))
        }
        _ => String::new(),
    };
    Ok(Cofre {
        existe,
        caminho: limpar(&j.str_ou_vazio("path")),
        tamanho: match j.get("size_bytes") {
            Some(Json::Num(n)) if *n > 0.0 => format!("{n:.0} bytes"),
            _ => String::new(),
        },
        permissao: j.str_ou_vazio("mode"),
        permissao_ok: j.bool("mode_ok").unwrap_or(false),
        kdf,
        kdf_fraco: j.bool("kdf_weak").unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVES: &str = r#"{
  "deployer": "0.7.2",
  "keys": [
    {"name": "github", "kind": "ED25519", "fingerprint": "SHA256:c4Z/1C1D9jm/7Am0aiGQEpp", "comment": "schematize:u@h", "public_path": "/home/u/.ssh/github.pub"}
  ]
}"#;

    const HOSTS: &str = r#"{
  "deployer": "0.7.2",
  "hosts": [
    {"alias": "srv-01", "host": "10.0.0.5", "port": 2222, "user": "deploy", "key_name": "github", "jump": null, "env": "prd", "mode": "readonly", "host_key_pinned": true, "fingerprint": "SHA256:hostkey", "boundary": "root", "server_side_boundary": true, "probed_at": 1788000000},
    {"alias": "lab", "host": "10.0.0.9", "port": 22, "user": "dev", "key_name": "lab", "jump": null, "env": "dev", "mode": "livre", "host_key_pinned": false, "fingerprint": null, "boundary": "sem", "server_side_boundary": false, "probed_at": 0}
  ]
}"#;

    const COFRE: &str = r#"{
  "deployer": "0.7.2", "exists": true, "path": "/home/u/.claude/schematize/deployer-cofre.bin",
  "size_bytes": 512, "mode": "600", "mode_ok": true,
  "kdf": {"m_cost": 65536, "t_cost": 3, "p_cost": 4}, "kdf_weak": false
}"#;

    // -----------------------------------------------------------------------
    // O varredor de segredo — a barreira do D5, virando teste.
    // -----------------------------------------------------------------------

    /// **O quê:** o que num texto denuncia material de chave privada ou segredo em claro.
    ///
    /// **Onde:** [`nenhum_modelo_carrega_segredo`].
    ///
    /// Ele existe porque a barreira não pode ser só "os tipos não têm o campo": um comentário
    /// de chave é texto livre digitado por gente, e alguém que cole um bloco PEM ali faz o
    /// modelo carregá-lo. O tipo não tem como impedir; o varredor tem.
    fn marcas_de_segredo(txt: &str) -> Vec<String> {
        let mut achados = Vec::new();
        for m in [
            "BEGIN OPENSSH PRIVATE KEY",
            "BEGIN RSA PRIVATE KEY",
            "BEGIN EC PRIVATE KEY",
            "PRIVATE KEY-----",
        ] {
            if txt.contains(m) {
                achados.push(format!("bloco PEM: {m}"));
            }
        }
        for pfx in ["ghp_", "github_pat_", "xoxb-", "sk-", "AKIA"] {
            if let Some(i) = txt.find(pfx) {
                let corpo = txt[i + pfx.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .count();
                if corpo >= 8 {
                    achados.push(format!("token com prefixo {pfx}"));
                }
            }
        }
        achados
    }

    /// **O D5, virando teste.** Nenhum dos três modelos carrega material de chave privada —
    /// nem quando o contrato traz um bloco PEM colado num campo de texto livre.
    #[test]
    fn nenhum_modelo_carrega_segredo() {
        // O caso rebuscado de propósito: um bloco PEM no comentário de uma chave.
        let hostil = CHAVES.replace(
            r#""comment": "schematize:u@h""#,
            r#""comment": "-----BEGIN OPENSSH PRIVATE KEY-----\nb3Blbn\n-----END""#,
        );
        let modelos = format!(
            "{:?}{:?}{:?}",
            ler_chaves(&hostil).unwrap(),
            ler_hosts(HOSTS).unwrap(),
            ler_cofre(COFRE).unwrap()
        );
        let achados = marcas_de_segredo(&modelos);
        assert!(achados.is_empty(), "SEGREDO NO MODELO — {}\n{modelos}", achados.join("; "));
    }

    /// **O varredor REPROVA segredo e ABSOLVE o que é público.** Guard nunca visto falhando é
    /// guard cego — e guard que reprova tudo é igualmente inútil, porque quem o vê reprovar um
    /// caso legítimo desliga o guard.
    #[test]
    fn o_varredor_reprova_segredo_e_absolve_o_publico() {
        for ruim in [
            "-----BEGIN OPENSSH PRIVATE KEY-----\nb3Blbn\n-----END OPENSSH PRIVATE KEY-----",
            "token: ghp_AbCdEf0123456789AbCdEf",
            "AKIAIOSFODNN7EXAMPLE",
        ] {
            assert!(!marcas_de_segredo(ruim).is_empty(), "devia reprovar: {ruim:?}");
        }
        for bom in [
            // O fingerprint é PÚBLICO — é o hash da pública, e é a confirmação que o D5 manda
            // usar. Um varredor que o reprovasse tornaria a tela inútil.
            "SHA256:c4Z/1C1D9jm/7Am0aiGQEpp/Cb/hPzlRHIxTWr6n6dU",
            "/home/u/.ssh/github.pub",
            "alias: skeleton, host: resource.example",
        ] {
            assert!(marcas_de_segredo(bom).is_empty(), "falso-positivo: {bom:?}");
        }
    }

    // -----------------------------------------------------------------------
    // As três telas
    // -----------------------------------------------------------------------

    /// A tela de chaves lê o contrato inteiro, e o fingerprint é o que ela mostra para
    /// confirmar identidade.
    #[test]
    fn a_tela_de_chaves_le_o_contrato() {
        let k = ler_chaves(CHAVES).unwrap();
        assert_eq!(k.len(), 1);
        assert_eq!(k[0].nome, "github");
        assert_eq!(k[0].algoritmo, "ED25519");
        assert!(k[0].fingerprint.starts_with("SHA256:"));
        assert!(k[0].caminho_publica.ends_with(".pub"), "o caminho é o da PÚBLICA");
    }

    /// **Lista vazia é FATO, e ilegível é `Err`.** Quem não tem chave precisa ver "gere a
    /// primeira"; quem não conseguiu falar com o deployer precisa ver o motivo. Achatar os dois
    /// faria a tela afirmar o que não sabe.
    #[test]
    fn lista_vazia_e_fato_e_ilegivel_e_erro() {
        assert!(ler_chaves(r#"{"keys":[]}"#).unwrap().is_empty());
        assert!(ler_chaves("{}").unwrap().is_empty());
        for lixo in ["", "isto nao e json", r#"{"keys":["#] {
            assert!(ler_chaves(lixo).is_err(), "{lixo:?}");
            assert!(ler_hosts(lixo).is_err(), "{lixo:?}");
            assert!(ler_cofre(lixo).is_err(), "{lixo:?}");
        }
    }

    /// Os slugs estáveis viram texto de tela, e o `prd` é marcado — errar em produção custa
    /// mais que errar em desenvolvimento, e a tela tem de deixar isso visível.
    #[test]
    fn os_slugs_viram_texto_e_a_producao_e_marcada() {
        let h = ler_hosts(HOSTS).unwrap();
        assert_eq!(h[0].destino, "deploy@10.0.0.5:2222");
        assert_eq!(h[0].ambiente, "produção");
        assert!(h[0].producao);
        assert_eq!(h[0].modo, "só leitura");
        assert!(!h[1].producao, "`dev` não é produção");
        assert_eq!(h[1].modo, "livre");
    }

    /// **A frase do host SEM fronteira é a que importa.** "sem" sozinho é informação que não
    /// informa: a pessoa precisa saber que ali só a política deste cliente protege, e que isso
    /// pega acidente, não intenção.
    #[test]
    fn a_ausencia_de_fronteira_e_dita_por_extenso() {
        let h = ler_hosts(HOSTS).unwrap();
        assert!(h[0].fronteira_servidor);
        assert!(h[0].fronteira.contains("sshd"), "{}", h[0].fronteira);

        assert!(!h[1].fronteira_servidor);
        assert!(h[1].fronteira.contains("SEM fronteira"), "{}", h[1].fronteira);
        assert!(h[1].fronteira.contains("só a política"), "{}", h[1].fronteira);
        assert_ne!(h[1].fronteira, "sem", "o slug cru não informa nada");
    }

    /// Host sem host key pinada: `confiado` falso e fingerprint vazio andam juntos. Dizer que
    /// está pinada sem ter fingerprint seria mentir sobre a única confirmação que a tela tem.
    #[test]
    fn confiado_e_fingerprint_andam_juntos() {
        let h = ler_hosts(HOSTS).unwrap();
        assert!(h[0].confiado && !h[0].fingerprint.is_empty());
        assert!(!h[1].confiado && h[1].fingerprint.is_empty());
    }

    /// A tela do cofre fala do ARQUIVO — e o custo do KDF sai em MiB, que é como o Argon2 o
    /// documenta. Em KiB seria um número de seis dígitos que ninguém compara de cabeça.
    #[test]
    fn a_tela_do_cofre_fala_do_arquivo_e_nao_do_conteudo() {
        let c = ler_cofre(COFRE).unwrap();
        assert!(c.existe);
        assert_eq!(c.permissao, "600");
        assert!(c.permissao_ok);
        assert!(c.kdf.contains("64 MiB"), "{}", c.kdf);
        assert!(c.kdf.contains("3 passagens"), "{}", c.kdf);
        assert!(!c.kdf_fraco);
    }

    /// **Cofre ausente é ESTADO, não erro.** É o estado inicial de toda instalação, e uma tela
    /// de erro ali seria lida como "quebrou".
    #[test]
    fn cofre_ausente_e_estado_e_nao_erro() {
        let c = ler_cofre(
            r#"{"exists": false, "path": "/home/u/x.bin", "size_bytes": 0,
                "mode": null, "mode_ok": false, "kdf": null, "kdf_weak": false}"#,
        )
        .expect("ausente é Ok, não Err");
        assert!(!c.existe);
        assert!(!c.caminho.is_empty(), "o caminho onde ELE ESTARIA ainda é útil");
        assert_eq!(c.permissao, "");
        assert_eq!(c.kdf, "");
        assert_eq!(c.tamanho, "", "zero bytes não é tamanho, é ausência");
    }

    /// Permissão frouxa é dita: `600` é o certo, e qualquer outra deixa o arquivo legível por
    /// mais gente do que devia.
    #[test]
    fn permissao_frouxa_e_dita() {
        let c = ler_cofre(
            COFRE
                .replace(r#""mode": "600", "mode_ok": true"#, r#""mode": "644", "mode_ok": false"#)
                .as_str(),
        )
        .unwrap();
        assert_eq!(c.permissao, "644");
        assert!(!c.permissao_ok);
    }

    /// **Entrada hostil não panica.** Uma janela que morre ao abrir é pior que uma tela vazia:
    /// a pessoa não vê nem a mensagem de erro.
    #[test]
    fn entrada_hostil_nao_panica() {
        // Tipos trocados em todos os campos.
        assert!(ler_chaves(r#"{"keys":[{"name":1,"kind":null,"fingerprint":[]}]}"#).is_ok());
        assert!(ler_hosts(r#"{"hosts":[{"port":"nao e numero","env":123}]}"#).is_ok());
        assert!(ler_cofre(r#"{"exists":"talvez","kdf":"nao e objeto"}"#).is_ok());
        // E o resultado é campo vazio, nunca dado inventado.
        let h = ler_hosts(r#"{"hosts":[{}]}"#).unwrap();
        assert_eq!(h[0].destino, "@:", "campos ausentes viram vazio, não palpite");
        assert!(!h[0].confiado, "sem o campo não se afirma que está confiado");
    }
}
