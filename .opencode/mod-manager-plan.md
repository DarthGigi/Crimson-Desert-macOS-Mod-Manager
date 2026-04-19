# Crimson Desert Mod Manager Plan

## Decisions

- Runtime storage is owned by Rust with SQLite.
- Drizzle and the TS-side SQLite runtime were removed.
- `mods/available` is archive-first library storage.
- V1 scope is parity-first with the existing Python manager.
- V1 import scope is folder-based `*.json` and `*.modpatch` files only.
- Patch-level toggles are deferred until after parity.

## Phases

1. Foundation
   - Replace the starter backend and UI scaffold.
   - Move persistence to Rust-owned SQLite under the Tauri app data directory.
   - Establish the Tauri command surface and game-path storage.

2. Game install management
   - Support `.app` bundle and `packages` directory resolution.
   - Implement saved path handling and macOS auto-detection.
   - Add install validation, writability reporting, and game launch support.

3. Binary patch engine
   - Port the PA checksum implementation.
   - Port PAMT parsing and file indexing.
   - Port full/simplified game-file resolution.
   - Port multi-file `0036/0.paz` and `0036/0.pamt` generation.
   - Port `meta/0.papgt` backup/update and restore behavior.
   - Use raw-block LZ4 decompress/recompress and 16-byte PAZ alignment.

4. Mod library workflow
   - Scan folders recursively for mod variants.
   - Import one chosen variant into an archive-first library.
   - Track imported mods, targets, counts, and enabled state in SQLite.
   - Support enable, disable, apply, restore, and reset flows.

5. Desktop UI
   - Build a single-page dashboard around game setup, import, library state, and overlay actions.
   - Keep the UI desktop-focused and practical, with visible apply results and status summaries.

6. Verification
   - Check Rust compilation and frontend type/build health continuously.
   - Compare the new manager’s behavior against the Python tool using real mods and a real install.

## Post-parity work

- Patch-level toggles
- Stronger conflict analysis and dry-run previews
- Archive import support
- Profiles and loadout management
