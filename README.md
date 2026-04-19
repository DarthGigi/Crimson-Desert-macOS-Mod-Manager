<p align="center">
  <img src="src-tauri/icons/app-icon.png" alt="Crimson Desert Mod Manager icon" width="160" height="160" />
</p>

<h1 align="center">Crimson Desert Mod Manager</h1>

A macOS-native mod manager for **Crimson Desert** built with **Tauri**, **Svelte**, **TypeScript**, **Tailwind**, **shadcn-svelte**, and **Rust**.

It supports JSON patch mods, precompiled overlays, browser/raw folder mods, archive imports, PATHC workflows, recovery tools, and advanced extraction utilities.

> [!IMPORTANT]
> This project was fully vibe coded by Codex, using OpenCode as the harness and GPT 5.4 as the model.
>
> That said, I do have real experience with the frontend stack used here, especially the UI-side technologies and patterns around Svelte, Tailwind, and component-based frontend work.

## What It Does

- Import mod sources from:
  - folders
  - `.zip`
  - `.7z`
  - `.rar`
- Manage mod types including:
  - JSON patch mods
  - precompiled overlay mods
  - browser/raw folder mods
  - language-targeted mods
  - ASI mods
  - BNK mods
  - binary patch mods (`.bsdiff`, `.xdelta`)
  - script-installer mods
- Reorder JSON mods and toggle individual patch groups
- Preview conflicts, overlaps, unresolved targets, and estimated overlay groups before apply
- Build fresh manager-owned overlay groups dynamically
- Restore vanilla state, reset active mods, and run a full `Fix Everything` cleanup
- Inspect and repack `0.pathc` from DDS folders
- Search, preview, and extract virtual files from the game archives
- Extract/decrypt and repack XML-style entries
- Save and apply mod profiles
- Run problem-mod isolation to narrow down crashes
- Export diagnostic reports

## Project Structure

- `src/`
  Svelte app, routed UI, shell, and client-side state
- `src-tauri/`
  Rust backend, Tauri config, bundling, icons, and native workflows
- `.github/workflows/`
  GitHub release workflow with macOS builds and attestations
- `.opencode/`
  internal notes and QA summary generated during development

## Main Pages

- `Overview`
- `Data Mods`
- `Language Mods`
- `Precompiled Mods`
- `ASI Mods`
- `External Mods`
- `Library`
- `Profiles`
- `Apply & Logs`
- `Tools`
- `Advanced`

## macOS Distribution Notes

- Minimum supported macOS version is set to **macOS 15**
- The app is configured with **ad-hoc signing**
- It is still expected that some users may need to **Gatekeeper bypass** the app on first launch
- The DMG has a custom background and drag-to-Applications layout

## Runtime Tooling

The app bundles `7z` for archive import so users do not need Homebrew just to import `.7z` / `.rar` mods.

Some advanced workflows may still depend on tools already present on the system or on optional bundled tools in the future, for example:

- `xdelta3` for `.xdelta`
- `python3` for Python-based script installers
- `/usr/bin/bspatch` for `.bsdiff`

## Development

Install dependencies:

```bash
pnpm install
```

Run the app in development:

```bash
pnpm tauri dev
```

Run checks:

```bash
pnpm check
cargo check --manifest-path src-tauri/Cargo.toml
```

Build the frontend only:

```bash
pnpm build
```

Build the macOS app bundle / DMG:

```bash
pnpm tauri build --bundles app,dmg
```

## Build Output

Typical macOS build outputs:

- `.app`
  `src-tauri/target/release/bundle/macos/Crimson Desert Mod Manager.app`
- `.dmg`
  `src-tauri/target/release/bundle/dmg/Crimson Desert Mod Manager_0.1.0_aarch64.dmg`

## Release Automation

There is a GitHub Actions workflow that:

- builds macOS bundles
- uploads release artifacts
- creates artifact attestations

See:

- `.github/workflows/release.yml`

## QA

The project includes automated validation and real-file tests against local Crimson Desert assets and downloaded mods.

For the latest summary, see:

- `.opencode/final-qa-summary.md`

## Credits

This project stands on a lot of community reverse-engineering, tooling, and format research. Credit goes to:

- [Crimson Desert - JSON Mod Manager](https://www.nexusmods.com/crimsondesert/mods/113)
- [lazorr410 / crimson-desert-unpacker](https://github.com/lazorr410/crimson-desert-unpacker)
- [993499094](https://www.nexusmods.com/profile/993499094) for the related ResHax discussion and reverse-engineering context:
  [post reference](https://reshax.com/topic/18908-need-help-extracting-paz-pamt-files-from-crimson-desert-blackspace-engine/page/2/?&_rid=3118#findComment-103796)
- [Crimson Desert mod 218](https://www.nexusmods.com/crimsondesert/mods/218)
- [faisalkindi / CrimsonDesert-UltimateModsManager](https://github.com/faisalkindi/CrimsonDesert-UltimateModsManager)
- [CallMeSlinky / Crimson-Desert-PATHC-Tools](https://github.com/CallMeSlinky/Crimson-Desert-PATHC-Tools)
- [Enki013 / Crimson-Desert-JSON-Mod-Manager-MacOS](https://github.com/Enki013/Crimson-Desert-JSON-Mod-Manager-MacOS)
- [MrIkso / CrimsonDesertTools](https://github.com/MrIkso/CrimsonDesertTools)

## License

MIT
