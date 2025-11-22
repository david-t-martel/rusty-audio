# Repository Guidelines

## Project Structure & Module Organization
Rusty Audio is a Cargo workspace with `rusty-audio-core` (DSP + device plumbing), `rusty-audio-desktop` (egui/eframe native shell), and `rusty-audio-web` (WASM + OAuth layer). The root `src/` binary orchestrates shared subsystems under `src/audio`, `src/ui`, `src/security`, and `src/testing`. Media lives in `assets/` + `static/`, browser bundles land in `dist/`, and experiments stay in `examples/`. Keep integration, UI, and wasm harnesses inside `tests/` (`e2e`, `ui`, `ui_kittest`, `wasm_*`, helpers).

## Build, Test, and Development Commands
- `just check` – runs `cargo check --all-targets`; use it as the first smoke test.
- `just run` – starts the egui shell in debug; switch to `just run-release` for profiling.
- `just serve-wasm` – trunk dev server on :8080 with live reload for OAuth/UI checks.
- `just build-wasm-release` – optimized wasm-pack build into `dist/pkg` for deployment.
- `just quality` – chains `fmt-check`, `lint`, and `test`, mirrors the CI gate.

## Coding Style & Naming Conventions
Format via `just fmt`, and enforce warning-free builds with `just lint` (`cargo clippy --all-targets -D warnings`). Modules/files stay snake_case, exported types use PascalCase, async helpers end in `_task`. Prefer `anyhow::Result` or domain errors over `unwrap`, and wrap long-running audio graph stages in `tracing` spans. UI presenters under `src/ui` should mutate shared state through managers, while WASM bridges in `rusty-audio-web/src` expose camelCase bindings via `wasm_bindgen`.

## Testing Guidelines
`just test` runs the full matrix (unit, integration, wasm). Reach for `just test-integration` to target `/tests`, `just test-wasm-headless` for Firefox/Chrome automation, and `just test-one suite::case` during tight loops. Property suites live in `src/property_tests.rs` and `tests/property_based_tests.rs` using `proptest`/`quickcheck`; place new generators beside the DSP change they validate. Benchmarks belong in `benches/` (Criterion HTML in `target/criterion`), and UI tweaks must extend `tests/ui_kittest` and ship screenshots.

## Commit & Pull Request Guidelines
Stick to Conventional Commits (`feat:`, `fix(scope):`, `chore:`) with ≤72-character subjects, matching `COMMIT_MESSAGE.md`. Run `just pre-commit` before pushing so fmt, lint, and tests mirror CI. In PRs summarize intent, list key changes, cite the commands you ran (paste snippets or screenshots), and link issues/spec docs—`pr_description.md` shows the expected tone. Call out any residual TODOs, include UI captures, and request review only once CI is green.

## Security & Configuration Tips
Keep OAuth secrets, API tokens, and credentials outside the repo; prefer OS keychains or ignored `.env.local` files. Windows contributors should run `scripts/setup-sccache-windows.ps1` once, confirm `rustup target add wasm32-unknown-unknown`, and verify `%CPAL_ASIO_DIR%` is set before exercising ASIO or WASM builds.
