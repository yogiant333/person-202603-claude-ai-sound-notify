# AI Sound Notify — Desktop Smoke Test Checklist

Run this checklist after every desktop release build.

## Basic launch

- [ ] Installer runs without admin prompt (currentUser install mode)
- [ ] App appears in Start Menu under **AI Sound Notify**
- [ ] Launching shows a 960x720 window with three source cards
- [ ] Tray icon appears in the system tray (expand the ^ overflow if hidden)

## Sound playback

- [ ] Each **Test Complete** button plays the rising tone for that source
- [ ] Each **Test Input** button plays the double beep
- [ ] Volume slider changes perceived loudness
- [ ] **Mute All** silences all test buttons

## Live server

- [ ] Status dot turns green within 2 s of launch
- [ ] Sending a real notification from any shell plays the correct sound:

  ```bash
  curl -s -X POST https://ainotify.keymantek.com:777/notify \
    -H 'Content-Type: application/json' \
    -d '{"source":"claude-code","event":"task_complete"}'
  ```

## Custom sounds

- [ ] Open **Sound Settings** panel → click the folder button next to any row
- [ ] Choose a local `.wav` or `.mp3`
- [ ] **Preview** plays that file
- [ ] A live notification plays that file
- [ ] Preference survives app restart

## Offline detection

- [ ] Disable the network adapter
- [ ] Within 60 s: triple rising-square alarm plays AND Windows toast shows **Server unreachable**
- [ ] Re-enable the network
- [ ] Within 30 s: ascending C-E-G chord plays AND toast shows **Server recovered**

## Tray behaviour

- [ ] Closing the window with ✕ hides it (tray icon remains, process still alive)
- [ ] Left-click tray icon toggles window visibility
- [ ] Right-click tray → **Show** / **Hide** / **Quit** each behave as expected
- [ ] **Quit** terminates the process (verify in Task Manager — no `ai-sound-notify.exe`)

## Autostart

- [ ] Enable **Start with Windows** toggle in Sound Settings
- [ ] Confirm the HKCU Run entry:

  ```cmd
  reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v ai-sound-notify
  ```
- [ ] Reboot Windows — app launches automatically (minimized to tray)
- [ ] Disable the toggle, reboot — app does NOT launch

## Single instance

- [ ] Launch a second copy while the first is running
- [ ] No second window appears; the existing window gains focus

## Uninstall

- [ ] Windows Settings → Apps → **AI Sound Notify** → Uninstall
- [ ] Installation folder removed from `%LOCALAPPDATA%\Programs\`
- [ ] Autostart registry entry removed

---

## Rebuilding after an icon change (gotcha)

Replacing `desktop/src-tauri/source-icon.png` + re-running `npx tauri icon`
regenerates `icons/icon.ico` but **Cargo's incremental build does NOT
re-embed the icon into `ai-sound-notify.exe`** on a subsequent `npm run build`.

The build script (`build.rs` → `tauri_build::build()`) generates a
`resource.rc` referencing `icons/icon.ico` and compiles it to `resource.lib`.
Cargo caches that `resource.lib` based on the build script's fingerprint, which
does not track the contents of `icons/icon.ico`. The regenerated `.ico` stays
on disk but the .exe keeps the old resource.

**Reliable fix — wipe the build-script output directory before rebuilding:**

```bash
# from WSL, repo root
rm -rf desktop/src-tauri/target/release/build/ai-sound-notify-* \
       desktop/src-tauri/target/release/ai-sound-notify.exe
(cd desktop && npm run build)
```

Or from PowerShell:

```powershell
Remove-Item -Recurse -Force desktop\src-tauri\target\release\build\ai-sound-notify-*
Remove-Item -Force desktop\src-tauri\target\release\ai-sound-notify.exe
cd desktop; npm run build
```

`cargo clean -p ai-sound-notify` is NOT enough — the cached `.lib` for the
build-script output is in a sibling directory it doesn't touch. `cargo clean`
(without `-p`) works but recompiles all ~400 deps (~3 min). The targeted
delete above only loses our own crate + resource step (~1 min).

**Verify the new icon got embedded:**

```powershell
Add-Type -AssemblyName System.Drawing
$i = [System.Drawing.Icon]::ExtractAssociatedIcon('desktop\src-tauri\target\release\ai-sound-notify.exe')
$i.ToBitmap().Save('C:\tmp\icon-check.png')
explorer.exe C:\tmp\icon-check.png
```

Compare the extracted PNG visually against the intended design before
uploading the installer.
