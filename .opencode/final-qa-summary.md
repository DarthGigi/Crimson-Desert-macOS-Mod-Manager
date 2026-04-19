# Final QA Summary

## Environment

- Game install: `/Users/gigi/Games/Crimson Desert.app`
- Downloaded mods corpus: `/Users/gigi/Downloads/CD Mods`

## Validation run

- `cargo check`
- `cargo test -- --nocapture`
- `pnpm check`
- `pnpm build`

## Real-file coverage

- JSON mod detection and sandbox apply
- Browser/raw folder mod detection and sandbox apply
- ZIP archive import detection
- 7Z archive import detection
- PATHC summary against real `meta/0.pathc`
- PATHC sandbox repack using a real DDS mod folder

## Feature coverage verified

- Routed sidebar workspace
- Dark/light theme toggle with dark default
- JSON patch toggles and load order
- Apply preview and conflict summary
- Language classification and targeting
- Precompiled and browser/raw import flows
- ZIP / 7Z / RAR archive import
- Recovery tools and `Fix Everything`
- Virtual file search, preview, and extraction
- PATHC inspection and DDS repack
- XML decrypt/extract and repack payload generation

## Notes

- XML in-place repack only proceeds when the new encrypted/compressed payload exactly matches the original compressed size.
- 7Z/RAR archive import depends on the system `7z` binary being available.
