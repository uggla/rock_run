# Changelog

All notable changes to this project will be documented in this file.

## [rockrun-0.3.0] - 2025-07-19

### 🚀 Features

- Bump to bevy 0.16.1 and all dependencies
- Bump to bevy 0.15.1
- Add an info log to show resolution

### 🐛 Bug Fixes

- Background level
- Rewrite of story plugin to deal with TextSpan
- Refactor of story plugin to deal with TextSpan done
- Controls
- Prevent crash when persistence file is outdated
- Properly hide level and lives on GameOver and GameFinished screens
- Force resize windows to fix wrong sizing
- Bug #2 avoid collisions with squirrels
- Fix clippy lints
- Do not remove the collected key if player looses a life
- Grounded not working with new rapier
- Unexpected crash due to region remove
- Tuned parameters and map layout to prevent player from getting stuck
- Wasm compilation
- Update wasm-bindgen-version
- Adjust font size

### 🚜 Refactor

- Bevy 0.15.1 #1
- Bevy 0.15.1 #2
- Bevy 0.15.1 #3

## [rockrun-0.2.2] - 2024-11-04

### 🐛 Bug Fixes

- Keyboard controls not aligned with documentation
- #1 not allowing to answer multiple times a correct answer

### 📚 Documentation

- Update README.md, add a TLDR section

## [rockrun-0.2.1] - 2024-10-31

### 🐛 Bug Fixes

- Fireballs hit in god mode
- Level 3 shader left and spawn on each level

## [rockrun-0.2.0] - 2024-10-30

### 🚀 Features

- Add git cliff configuration to generate futur change logs
- Add persistance for language selection
- Add 2 new questions
- Add a background shader for level 3
- Update dependencies serde, serde_json, thiserror

### 🐛 Bug Fixes

- Set default language to English
- Text in display menu not aligned properly on some platform
- Slight camera zoom to hide level boundaries

### 📚 Documentation

- Update TODO list
- Update README status
- Update documentation with debugging keys and storage info.
