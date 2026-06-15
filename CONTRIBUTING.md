# Contributing to Abencat Browser

Thanks for your interest! Abencat Browser (阿笨猫指纹浏览器) is an open-source
controller/launcher around the [CloakBrowser](https://github.com/CloakHQ/CloakBrowser)
fingerprint kernel. Contributions — code, docs, translations, bug reports — are welcome.

## Project layout

```
src-tauri/src/   Rust backend (models, store, crypto, proxy, launch, api, commands, browser)
src/             React/TS frontend (App.tsx, i18n.ts, components/)
mcp/             MCP server (cloak-mcp.mjs)
website/         landing page (browser.abencat.com)
docs/            multi-language READMEs + guides
scripts/         download-browser.{sh,ps1}
```

## Dev setup

Prereqs: **Rust** (stable) and **Node ≥ 18**.

```bash
# Fetch the fingerprint kernel (not bundled)
./scripts/download-browser.sh          # or scripts\download-browser.ps1 on Windows

npm install
npm run tauri:dev                      # run the desktop app (hot reload)

# Backend checks
cd src-tauri
cargo check                            # desktop (gui) build
cargo check --bin cloak-headless --no-default-features   # headless build
cargo test --lib                       # tests

# Frontend checks
npm run build                          # tsc + vite
```

## Guidelines

- **Match the surrounding style.** Backend is modular Rust; keep modules focused.
  Frontend strings should go through `src/i18n.ts` (zh/en required; other langs
  fall back to English).
- **Keep secrets & binaries out of git.** Never commit the kernel, build output,
  `controller-data/`, `.cloak_key`, `master.json`, or files in `release-assets/`
  (all already in `.gitignore`). GitHub rejects files > 100 MB.
- **Both build configs must pass:** `cargo check` (gui) and
  `cargo check --bin cloak-headless --no-default-features`, plus `tsc --noEmit`.
- **Adding a UI language?** Add it to `LANGS` in `src/i18n.ts` and fill the
  `EXTRA` dictionary (missing keys fall back to English).
- **Commits / PRs:** clear messages; one logical change per PR; describe what and why.

## Reporting issues

Use the issue templates. Include OS, version, steps to reproduce, and logs
(`journalctl -u cloak-headless` for the headless server). Please **redact proxies,
tokens and passwords**.

## License

By contributing you agree your contributions are licensed under the
[MIT License](LICENSE). The fingerprint kernel is © CloakHQ (CloakBrowser).
