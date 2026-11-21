# BUDGET — orçamento de memória da janela (piso de eficiência de runtime da casa)

Teto = **medido + folga ~30%**, e abaixo de 400 MiB (GUI). Subir teto exige medição nova e
justificativa escrita aqui. Teste que faz valer: `tests/orcamento_memoria.rs` (std-only; lê
`VmHWM` de `/proc/<pid>/status` com a janela aberta por 5 s). Pula com `eprintln` explícito se
não há `/proc` ou display (`DISPLAY`/`WAYLAND_DISPLAY`) — janela não abre sem display.

| Operação | RSS pico alvo (teto) | Medido antes → depois | Como medir |
|---|---|---|---|
| Janela aberta (telas Chaves SSH/Hosts, 5 s) | 160 MiB (build debug do teste) | 118,5 MiB debug · 98,2 MiB release (22 threads) → idem, sem mudança | `cargo test --test orcamento_memoria -- --nocapture`; release: poll de `VmHWM` do `~/.cargo/bin/schematize-deployer-gui` |
| Idle após a primeira pintura | (mesmo teto) | ~98 MiB release (HWM e RSS iguais: não há pico transitório) | idem |

Binário release instalado (v0.1.2): 22,65 MB. `Cargo.lock`: 594 pacotes (inclui alvos
Windows/macOS que não entram no build Linux).

## Diagnóstico (sem mudança de código — razão registrada)

- `slint` já está com `default-features = false` e só `std`, `compat-1-2`, `backend-winit`,
  `renderer-femtovg`, `accessibility`. Não há feature sobrando: `accessibility` é piso de a11y
  (WCAG 2.2 AA) e `backend-winit`/`femtovg` são o caminho de render. Os ~98 MiB são o piso do
  toolkit (contexto GL, atlas de glifos, winit, AT-SPI); só caem trocando de toolkit, o que é
  decisão de ADR e não "mudança mínima de comportamento idêntico".
- A janela é casca fina: lê JSON do binário headless por `Command::output()` (saída pequena,
  ~KiB) e não mantém cache, canal ou thread própria. Sem polling (`Timer`/`sleep`) no código.
- Duplicatas (`syn`, `thiserror`, `nom`, `windows-sys`, `objc2*`, …) vêm do ecossistema
  slint/winit e de alvos não-Linux; não são alcançáveis daqui.
