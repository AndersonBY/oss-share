# Installer Options Design

**Date:** 2026-04-08

## Goal

Improve the NSIS installer UX so users can choose whether to create a desktop shortcut before installation and choose whether to run OSS Share on the finish page.

## Scope

- Add a pre-install options page with a `创建桌面快捷方式` checkbox.
- Keep creating the Start Menu shortcut by default.
- Add a finish-page `运行 OSS Share` checkbox.
- Default values:
  - desktop shortcut: unchecked
  - run app after install: checked

## Constraints

- The app must not be launched elevated from the installer process.
- The finish-page run action must start OSS Share as the normal logged-in user.
- Existing sparse package registration and certificate installation flow must stay intact.

## Recommended Approach

Use a small custom NSIS options page for the desktop-shortcut choice and the standard MUI finish-page run checkbox for post-install launch.

## Data Flow

1. User reaches a custom options page before files are copied.
2. Installer stores whether desktop shortcut should be created.
3. Install section creates the Start Menu shortcut unconditionally.
4. Install section creates the desktop shortcut only if selected.
5. Finish page shows `运行 OSS Share` checked by default.
6. If the user leaves it checked, NSIS launches OSS Share via a non-elevated shell path.

## Uninstall Behavior

- Remove Start Menu shortcut.
- Remove desktop shortcut if it exists.

## Files to Change

- `installer/nsis/installer.nsi`

## Testing Strategy

- Build installer with `makensis`.
- Install once with desktop shortcut unchecked.
- Install once with desktop shortcut checked.
- Verify finish-page run checkbox is present and checked by default.
- Verify launched app is usable for right-click uploads.
