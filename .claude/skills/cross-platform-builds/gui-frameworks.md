# GUI Frameworks for Cross-Platform Applications

For applications with user interfaces, higher-level frameworks handle native windowing, rendering, and event handling across platforms.

## Framework Overview

| Framework | Frontend Tech | Platforms | Bundle Size | Best For |
|:----------|:--------------|:----------|:------------|:---------|
| **Tauri** | Web (HTML/CSS/JS) | Desktop, Mobile | Small (uses OS webview) | Web devs, existing web skills |
| **Dioxus** | Rust (React-like) | Desktop, Mobile, Web, TUI | Varies by renderer | Full Rust stack, code reuse |
| **Slint** | Declarative DSL | Desktop, Embedded | Small | Resource-constrained devices |
| **egui** | Immediate mode | Desktop, Web | Small | Tools, debugging UIs |

---

## Tauri

Use established web technologies (HTML/CSS/JS with React, Vue, Svelte) for UI, with Rust handling backend logic.

### Key Features

- **Tiny bundles** via OS native webview (no bundled Chromium)
- **Strong security model** with fine-grained permissions
- **IPC between JS frontend and Rust backend**
- **Mobile support** (iOS/Android) in Tauri 2.0

### Quick Start

```bash
# Create new project
cargo install create-tauri-app
cargo create-tauri-app

# Or add to existing web project
cd my-web-app
cargo install tauri-cli
cargo tauri init
```

### Project Structure

```
my-app/
├── src-tauri/           # Rust backend
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── tauri.conf.json
└── src/                 # Web frontend
    ├── index.html
    └── main.js
```

### Rust Backend Example

```rust
// src-tauri/src/main.rs
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

### Frontend Invocation

```javascript
import { invoke } from '@tauri-apps/api/tauri';

const greeting = await invoke('greet', { name: 'World' });
console.log(greeting); // "Hello, World!"
```

### Cross-Platform Build

```bash
# Build for current platform
cargo tauri build

# Build for specific target
cargo tauri build --target aarch64-apple-darwin

# Supported targets configured in tauri.conf.json
```

---

## Dioxus

Write your entire UI in Rust using a React-like paradigm. Single codebase targets web, desktop, mobile, and TUI.

### Key Features

- **Familiar React patterns** (hooks, components, state)
- **Hot reloading** during development
- **True cross-platform** from one codebase
- **No JavaScript required** for desktop apps

### Quick Start

```bash
cargo install dioxus-cli
dx new my-app
cd my-app
dx serve  # Development server with hot reload
```

### Component Example

```rust
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            h1 { "Counter: {count}" }
            button { onclick: move |_| count += 1, "Increment" }
            button { onclick: move |_| count -= 1, "Decrement" }
        }
    }
}
```

### Renderer Selection

```toml
# Cargo.toml

# Desktop (native window)
[dependencies]
dioxus = { version = "0.5", features = ["desktop"] }

# Web (WebAssembly)
dioxus = { version = "0.5", features = ["web"] }

# Mobile (iOS/Android)
dioxus = { version = "0.5", features = ["mobile"] }

# TUI (terminal)
dioxus = { version = "0.5", features = ["tui"] }
```

### Build Commands

```bash
# Desktop
dx build --release

# Web (generates WASM)
dx build --platform web --release

# Bundle for distribution
dx bundle
```

---

## Slint

Declarative UI language designed for resource-constrained devices and desktop.

### Key Features

- **Own markup language** (.slint files) that's intuitive
- **Efficient rendering** - suitable for embedded
- **Compile-time UI verification**
- **Multiple language bindings** (Rust, C++, JS)

### Quick Start

```bash
cargo new slint-app
cd slint-app
```

```toml
# Cargo.toml
[dependencies]
slint = "1.3"
```

### UI Definition

```slint
// ui/main.slint
export component MainWindow inherits Window {
    in-out property <int> counter: 0;

    VerticalLayout {
        Text { text: "Counter: \{counter}"; }
        Button {
            text: "Increment";
            clicked => { counter += 1; }
        }
    }
}
```

### Rust Integration

```rust
slint::include_modules!();

fn main() {
    let main_window = MainWindow::new().unwrap();
    main_window.run().unwrap();
}
```

---

## egui

Immediate mode GUI—rebuild UI every frame. Great for tools and debugging interfaces.

### Key Features

- **No retained state** - simple mental model
- **Easy integration** with game engines
- **Portable** - web via egui_web
- **Fast iteration** during development

### Quick Start (with eframe)

```toml
[dependencies]
eframe = "0.24"
```

```rust
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

#[derive(Default)]
struct MyApp {
    counter: i32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Counter");
            ui.label(format!("Value: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
        });
    }
}
```

---

## Framework Decision Guide

```
Do you have existing web skills / want web tech?
├── Yes → Tauri
└── No → Continue

Do you want a React-like Rust experience?
├── Yes → Dioxus
└── No → Continue

Is this for embedded / resource-constrained?
├── Yes → Slint
└── No → Continue

Is this a dev tool / debugging UI?
├── Yes → egui
└── No → Consider Tauri or Dioxus
```

## Cross-Platform Build Notes

All frameworks support building for multiple platforms, but remember:

- **macOS builds** require macOS host or CI runner (code signing, SDK)
- **iOS builds** require Xcode on macOS
- **Android builds** require Android SDK/NDK
- **Windows builds** can be done from Linux via cross-compilation

For production releases, use CI/CD with native runners per platform when possible.
