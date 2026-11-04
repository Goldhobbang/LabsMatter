# A5 Manual Runtime Verification Instructions

- Owner: A0
- Date: 2026-11-04
- For: User/Operator (no browser in CLI)

## Setup
1. Serve web/ directory (e.g., `python -m http.server` or similar)
2. Open in modern browser with WebGPU/WebGL2 support
3. Open DevTools Console

## Tests
1. **Init**: Console shows "Using WASM simulation (real module)". No WASM load errors.
2. **Paint sand**: Select sand, paint → falls under gravity
3. **Paint water**: Flows horizontally/settles
4. **Paint wall**: Static, blocks
5. **Erase**: Removes cells
6. **Reset**: Clears world
7. **Controls**: Play/pause/step/speed work
8. **Save/load**: Save blob, reload, verify state
9. **Performance**: Check FPS in HUD
10. **Errors**: No console errors

## Report
If issues found, report to supervisor with console logs/screenshots. A8 can re-verify.
