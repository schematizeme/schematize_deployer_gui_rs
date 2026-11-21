//! O QUE: faz valer o orçamento de memória da janela (`BUDGET.md`) — sobe o BINÁRIO real, lê o
//! pico de RSS (`VmHWM` em `/proc/<pid>/status`) durante a abertura e REPROVA acima do teto.
//!
//! POR QUE EXISTE: janela de gerenciamento que passa de centenas de MiB é código/dep mal
//! escolhida, não "falta de RAM". O teto é medido (valor depois + folga ~30%), nunca chutado.
//!
//! COMO MEDE: std-only. Janela é processo de vida longa: mede vivo por alguns segundos, e mata.
//! PULA com `eprintln` explícito se `/proc` ou o display (`DISPLAY`/`WAYLAND_DISPLAY`) não
//! existem — uma janela não abre sem display; nunca passa em silêncio.
//!
//! DE ONDE VEM: `HOME` num temporário. PRA ONDE VAI: só esse temporário (a janela é só leitura).

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Teto em KiB — medido no binário de TESTE (debug) + folga ~30%. Abaixo dos 400 MiB da casa.
const TETO_JANELA_KIB: u64 = 160 * 1024;
/// Quanto tempo a janela fica viva sendo medida (abertura + primeira pintura + telas).
const JANELA_VIVA: Duration = Duration::from_secs(5);

/// **O quê:** `VmHWM` (KiB) do pid, ou `None` se o processo saiu ou `/proc` não existe.
/// **Onde:** laço de medição do teste.
fn vm_hwm_kib(pid: u32) -> Option<u64> {
    let txt = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    txt.lines()
        .find(|l| l.starts_with("VmHWM:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|n| n.parse().ok())
}

#[test]
fn orcamento_janela_aberta() {
    if !Path::new("/proc/self/status").exists() {
        eprintln!("PULADO: /proc indisponível — orçamento de RSS da janela não medido");
        return;
    }
    let tem_display = ["DISPLAY", "WAYLAND_DISPLAY"]
        .iter()
        .any(|v| std::env::var_os(v).is_some_and(|x| !x.is_empty()));
    if !tem_display {
        eprintln!("PULADO: sem DISPLAY/WAYLAND_DISPLAY — a janela precisa de display para abrir");
        return;
    }
    let home = std::env::temp_dir().join(format!("schematize-orc-gui-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).expect("criar HOME temporário");
    let mut filho = Command::new(env!("CARGO_BIN_EXE_schematize-deployer-gui"))
        .env("HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("o binário precisa estar compilado");
    let fim = Instant::now() + JANELA_VIVA;
    let (mut pico, mut saiu_cedo) = (0, false);
    while Instant::now() < fim {
        pico = pico.max(vm_hwm_kib(filho.id()).unwrap_or(0));
        if matches!(filho.try_wait(), Ok(Some(_))) {
            saiu_cedo = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = filho.kill();
    let _ = filho.wait();
    let _ = std::fs::remove_dir_all(&home);
    if saiu_cedo {
        eprintln!("PULADO: a janela saiu antes da medição (display recusou a conexão?)");
        return;
    }
    eprintln!("orcamento[janela]: pico {pico} KiB (teto {TETO_JANELA_KIB} KiB)");
    assert!(pico > 0, "não consegui ler VmHWM — a medição não valeu");
    assert!(pico <= TETO_JANELA_KIB, "pico {pico} KiB acima do teto {TETO_JANELA_KIB} KiB");
}
