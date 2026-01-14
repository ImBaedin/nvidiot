# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nvidiot is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Development Commands

```bash
# Start development (runs both frontend and Tauri backend with hot reload)
bun run tauri dev

# Build production app
bun run tauri build

# TypeScript type checking
tsc

# Frontend-only dev server (port 1420)
bun run dev
```

## Architecture

**Frontend (src/):**
- React 19 with TypeScript
- Vite dev server on port 1420
- Uses `@tauri-apps/api/core` `invoke()` to call Rust commands

**Backend (src-tauri/):**
- Rust with Tauri v2
- Commands defined with `#[tauri::command]` in `src-tauri/src/lib.rs`
- Commands registered in `invoke_handler(tauri::generate_handler![...])`
- Entry point: `src-tauri/src/main.rs` calls `nvidiot_lib::run()`

**Frontend-Backend Communication:**
```typescript
// Frontend: call Rust command
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { arg1, arg2 });
```
```rust
// Backend: define command
#[tauri::command]
fn command_name(arg1: &str, arg2: i32) -> String { ... }
```
