# Agent.md — ashell project context (AI agent guide)

> Этот файл — подробный контекст для AI-агента, работающего с проектом ashell.
> Он дополняет CLAUDE.md и предназначен для быстрой ориентации в коде.

---

## Быстрые команды

```bash
make check    # cargo fmt --check + cargo check + clippy -D warnings (обязательно перед push)
make build    # cargo build --release
make fmt      # cargo fmt
make start    # build + ./target/release/ashell
make install  # build + sudo cp binary to /usr/bin
```

**Логи:** `/tmp/ashell/` (ротация по дням, 7 дней хранения)  
**Конфиг:** `~/.config/ashell/config.toml` (или `--config-path <path>`)

---

## Обзор проекта

**ashell** — Wayland status bar (панель состояния) для Hyprland и Niri.  
Написан на **Rust** (edition 2024, MSRV 1.89) с использованием **iced** (кастомный форк `iced_layershell` с поддержкой multi-window, wayland, wgpu).  
Архитектура: **Elm** (Model → Update → View).

- Версия: `0.8.0`
- Лицензия: MIT
- iced fork: `git = "https://github.com/MalpenZibo/iced_layershell"`, tag `v0.1.3`

---

## Архитектура: Elm pattern

```
App (src/app.rs)
 ├── state: всё состояние приложения в одной структуре
 ├── Message enum: все возможные события (ConfigChanged, Updates, Workspaces, ...)
 ├── update(&mut self, message) -> Task<Message>: обновление состояния
 ├── view(&self, id: SurfaceId) -> Element<Message>: рендеринг
 └── subscription(&self) -> Subscription<Message>: подписки на события
```

### Ключевые типы App

```rust
pub struct App {
    config_path: PathBuf,
    pub theme: AshellTheme,
    pub general_config: GeneralConfig,
    pub outputs: Outputs,          // многомониторность
    pub custom: HashMap<String, Custom>,
    pub updates: Option<Updates>,
    pub workspaces: Workspaces,
    pub window_title: WindowTitle,
    pub system_info: SystemInfo,
    pub keyboard_layout: KeyboardLayout,
    pub keyboard_submap: KeyboardSubmap,
    pub tray: TrayModule,
    pub tempo: Tempo,              // clock/calendar/weather
    pub privacy: Privacy,
    pub settings: Settings,
    pub media_player: MediaPlayer,
    pub notifications: Notifications,
    pub visible: bool,             // SIGUSR1 toggle
}
```

---

## Структура директорий

```
src/
├── main.rs          # точка входа: clap args, logger, iced LayerShell setup
├── app.rs           # App struct, Message enum, update(), view(), subscription()
├── config.rs        # TOML-десериализация конфига (~1200 строк)
├── theme.rs         # AshellTheme: Islands/Solid/Gradient стили
├── outputs.rs       # Outputs: управление окнами на мониторах (multi-window)
├── components/      # переиспользуемые UI-компоненты
│   ├── button.rs
│   ├── centerbox.rs          # кастомный виджет 3-колонки (left/center/right)
│   ├── format_indicator.rs
│   ├── icons.rs              # константы иконок Nerd Fonts (\u{XXXX})
│   ├── menu.rs               # MenuType enum, menu UI
│   ├── menu_wrapper.rs
│   ├── module_group.rs
│   ├── module_item.rs        # кнопка модуля с поддержкой press/right-press/scroll
│   ├── password_dialog.rs
│   ├── position_button.rs    # кастомный виджет (tracking button position)
│   ├── quick_setting_button.rs
│   ├── slider_control.rs
│   └── sub_menu_wrapper.rs
├── modules/         # UI-модули (каждый: view() + subscription())
│   ├── mod.rs                # маршрутизация ModuleName → view/subscription
│   ├── custom_module.rs      # CustomModule: Button / Text типы
│   ├── keyboard_layout.rs
│   ├── keyboard_submap.rs
│   ├── media_player.rs       # MPRIS-медиаплеер
│   ├── notifications.rs      # D-Bus notifications + toast popups
│   ├── privacy.rs            # mic/camera/screenshare индикаторы
│   ├── system_info.rs        # CPU/RAM/Disk/Temp/Network
│   ├── tempo.rs              # clock + calendar + weather (~49кБ, самый большой)
│   ├── tray.rs               # SystemTray
│   ├── updates.rs            # OS updates индикатор
│   ├── window_title.rs
│   ├── workspaces.rs
│   └── settings/             # панель настроек
│       ├── mod.rs            # основной Settings модуль (~34кБ)
│       ├── audio.rs          # аудио источники/выходы
│       ├── bluetooth.rs
│       ├── brightness.rs
│       ├── network.rs        # WiFi (NetworkManager + IWD)
│       └── power.rs          # питание, профили, power menu
└── services/        # backend: D-Bus, IPC, системная интеграция
    ├── mod.rs                # трейты ReadOnlyService, Service, ServiceEvent
    ├── audio.rs              # PulseAudio/PipeWire (libpulse-binding + pipewire)
    ├── bluetooth/
    ├── brightness.rs         # D-Bus brightness
    ├── compositor/           # Hyprland/Niri IPC абстракция
    │   ├── mod.rs            # авто-детект, broadcast broadcaster
    │   ├── hyprland.rs       # Hyprland IPC
    │   ├── niri.rs           # Niri IPC (niri-ipc crate)
    │   └── types.rs          # CompositorService, CompositorState, CompositorEvent
    ├── idle_inhibitor.rs
    ├── logind.rs             # logind D-Bus (для ResumeFromSleep)
    ├── mpris/                # MPRIS2 медиаплеер
    ├── network/              # NetworkManager + IWD backends
    ├── notifications/        # D-Bus notification server
    ├── privacy.rs
    ├── throttle.rs           # утилита для throttle подписок
    ├── tray/                 # system tray (StatusNotifierItem)
    └── upower/               # battery, peripheral devices
```

---

## Трейты сервисов

```rust
// Только чтение (подписка на события)
pub trait ReadOnlyService: Sized {
    type UpdateEvent;
    type Error: Clone;
    fn update(&mut self, event: Self::UpdateEvent);
    fn subscribe() -> Subscription<ServiceEvent<Self>>;
}

// С командами (чтение + запись)
pub trait Service: ReadOnlyService {
    type Command;
    fn command(&mut self, command: Self::Command) -> Task<ServiceEvent<Self>>;
}

pub enum ServiceEvent<S: ReadOnlyService> {
    Init(S),
    Update(S::UpdateEvent),
    Error(S::Error),
}
```

---

## Модули: как они устроены

Каждый модуль в `src/modules/` следует паттерну:

```rust
// Структура состояния модуля
pub struct MyModule { ... }

// Сообщения модуля
#[derive(Debug, Clone)]
pub enum Message { ... }

// Возможные действия (для модулей с командами)
pub enum Action {
    None,
    Command(Task<Message>),
    CloseMenu(SurfaceId),
    ...
}

impl MyModule {
    pub fn new(config: MyModuleConfig) -> Self { ... }
    pub fn update(&mut self, message: Message) -> Action { ... }  // или Task
    pub fn view<'a>(&'a self, theme: &'a AshellTheme) -> Element<'a, Message> { ... }
    pub fn subscription(&self) -> Subscription<Message> { ... }
    // Опционально для модулей с меню:
    pub fn menu_view<'a>(&'a self, theme: &'a AshellTheme) -> Element<'a, Message> { ... }
}
```

Маршрутизация в `App::update()` — каждый вариант `Message` вызывает `module.update(msg)` и маппит результат через `.map(Message::ModuleName)`.

---

## Конфигурация (config.rs)

**Файл:** `~/.config/ashell/config.toml`  
**Hot-reload:** inotify watch → `Message::ConfigChanged` → `App::refresh_config()`

### Основные секции конфига

| Секция | Тип | Описание |
|--------|-----|----------|
| `position` | `Position` | `Top` / `Bottom` |
| `layer` | `Layer` | `Top` / `Bottom` / `Overlay` |
| `outputs` | `Outputs` | какие мониторы показывать |
| `modules.left/center/right` | `Vec<ModuleDef>` | расположение модулей |
| `appearance` | `Appearance` | стиль, цвета, шрифт, scale_factor |
| `[CustomModule]` | `Vec<CustomModuleDef>` | кастомные модули |
| `updates` | `Option<UpdatesModuleConfig>` | OS updates |
| `workspaces` | `WorkspacesModuleConfig` | видимость, именование |
| `window_title` | `WindowTitleConfig` | mode: Title/Class/InitialTitle/InitialClass |
| `system_info` | `SystemInfoModuleConfig` | CPU/RAM/Disk/Temp пороги |
| `notifications` | `NotificationsModuleConfig` | toast позиция, timeout |
| `tempo` | `TempoModuleConfig` | формат часов, таймзоны, погода |
| `settings` | `SettingsModuleConfig` | команды питания, индикаторы |
| `media_player` | `MediaPlayerModuleConfig` | формат, длина title |
| `keyboard_layout` | `KeyboardLayoutModuleConfig` | кастомные метки |

### Стили appearance

- **Islands** (default): floating islands, нет фона панели
- **Solid**: сплошной фон с opacity
- **Gradient**: градиент от фона к прозрачному

### ModuleDef и ModuleName

```rust
// В конфиге можно задавать одиночные модули или группы:
// left = ["Workspaces", ["Clock", "Battery"]]

pub enum ModuleDef {
    Single(ModuleName),
    Group(Vec<ModuleName>),
}

pub enum ModuleName {
    Custom(String),
    Updates, Workspaces, WindowTitle, SystemInfo,
    KeyboardLayout, KeyboardSubmap, Tray, Tempo,
    Privacy, MediaPlayer, Settings, Notifications,
}
```

---

## Compositor абстракция

`CompositorService` авто-определяет бэкенд при старте:
1. Проверяет `HYPRLAND_INSTANCE_SIGNATURE` env → Hyprland
2. Иначе проверяет наличие Niri socket → Niri

Используется `OnceLock<Option<CompositorChoice>>` для кэширования выбора.  
Broadcast channel (capacity 64) рассылает события всем подписчикам.

---

## Multi-monitor (outputs.rs)

`Outputs` управляет iced `LayerShell` окнами:
- Основная панель (main layer) — по одному на монитор
- Menu layer — popup меню (открывается по клику)
- Toast layer — уведомления (отдельное окно)

`HasOutput` enum: `Main`, `Menu(Option<(MenuType, ButtonUIRef)>)`, `Toast`

---

## Иконки и шрифты (build.rs + components/icons.rs)

`build.rs` при сборке:
1. Извлекает все `\u{XXXX}` из `src/components/icons.rs`
2. Создаёт subset Nerd Fonts только с используемыми глифами
3. Выходные файлы: `target/generated/SymbolsNerdFont-Regular-Subset.ttf` и `*Mono*`

**Для добавления новой иконки:** найти код на [nerdfonts.com](https://www.nerdfonts.com/cheat-sheet), добавить константу в `src/components/icons.rs`.

---

## Сигналы и клавиши

- **SIGUSR1** → `Message::ToggleVisibility` (скрыть/показать панель)
- **ESC** → `Message::CloseAllMenus` (если `enable_esc_key = true` в конфиге)

---

## Качество кода

- **Clippy:** `cargo clippy --all-features -- -D warnings` (zero warnings в CI)
- **Fmt:** `cargo fmt --all -- --check` (enforced в CI)
- **Нет тестов** — контроль через clippy + сборку
- Перед push: **обязательно** `make check`

---

## Commit conventions

```
<type>(<optional-scope>): <subject>

Типы: feat, fix, docs, chore, style, refactor, perf, ci
Примеры:
  feat(system_info): add disk usage indicator
  fix(network): handle iwd connection timeout
  refactor(tempo): extract weather fetching logic
```

---

## Ключевые зависимости

| Crate | Назначение |
|-------|-----------|
| `iced_layershell` | GUI framework (iced fork с Wayland/wgpu) |
| `zbus` | D-Bus (async, tokio) |
| `hyprland` | Hyprland IPC |
| `niri-ipc` | Niri IPC |
| `libpulse-binding` | PulseAudio |
| `pipewire` | PipeWire |
| `sysinfo` | CPU/RAM/Disk/сеть |
| `inotify` | file watch для hot-reload конфига |
| `reqwest` | HTTP (погода в tempo модуле) |
| `chrono` + `chrono-tz` | дата/время/таймзоны |
| `regex` | regex в custom modules |
| `clap` | CLI args |
| `toml` + `serde` | конфиг |
| `allsorts` | font subsetting (build.rs) |

**Системные зависимости:**  
`libxkbcommon`, `libwayland`, `libpipewire-0.3`, `libpulse`, `dbus`, `udev`, `pkg-config`, `clang/llvm`

---

## Паттерны, которые нужно соблюдать

### Добавление нового модуля

1. Создать `src/modules/my_module.rs` со структурой, `Message`, `Action`, методами `new/update/view/subscription`
2. Добавить `pub mod my_module;` в `src/modules/mod.rs`
3. Добавить вариант `MyModule` в `ModuleName` (`config.rs`)
4. Добавить поле в `App` struct (`app.rs`) + инициализацию в `App::new()`
5. Добавить вариант в `Message` enum (`app.rs`)
6. Добавить обработку в `App::update()` (`app.rs`)
7. Добавить view в `get_module_view()` (`modules/mod.rs`)
8. Добавить subscription в `get_module_subscription()` (`modules/mod.rs`)
9. Добавить config-тип в `config.rs` + поле в `Config`
10. Если модуль имеет меню — добавить `MenuType::MyModule` (`components/menu.rs`)

### Добавление нового сервиса

1. Создать `src/services/my_service.rs`
2. Реализовать `ReadOnlyService` и/или `Service`
3. Добавить `pub mod my_service;` в `src/services/mod.rs`
4. Использовать в нужном модуле через `MyService::subscribe()`

### Работа с D-Bus (zbus)

```rust
// Паттерн: proxy через zbus async
use zbus::Connection;
let conn = Connection::system().await?;
let proxy = MyProxy::new(&conn).await?;
```

### Работа с `Task` (async операции)

```rust
// Task::perform для одноразовых async операций
Task::perform(
    async { fetch_data().await },
    |result| Message::DataFetched(result),
)

// Task::none() для синхронных обновлений без side-effects
```

---

## Документация

- **Пользовательская:** `website/` (Docusaurus) → https://malpenzibo.github.io/ashell/
- **Developer guide:** `docs/` (mdbook) → https://malpenzibo.github.io/ashell/dev-guide/
- Наиболее полезные для AI: architecture overview, anatomy-of-a-module, build.rs font subsetting
- Config reference полезен при review, не при реализации

---

## Контекст среды пользователя

- **OS:** ALT Linux (Sisyphus), Wayland, **niri** compositor
- **Hardware:** Minisforum UM780 XTX (AMD Ryzen 7 7840HS, Radeon 780M, 64GB RAM)
- **Display:** 3440x1440@165Hz
- Пользователь использует **Niri** — тестирование и отладка ведётся под Niri

## ALT Linux форк: специфика

### Пакетный менеджер

ALT Linux использует **RPM + APT** (`apt-get`). Команды для Updates модуля:

```bash
# Проверка обновлений (без реальной установки)
sudo apt-get dist-upgrade --simulate 2>/dev/null

# Формат строк вывода Inst:
# Inst <pkg> [<old>:<epoch>] (<new>:<epoch> Sisyphus:...)
```

### Скрипт check-updates

Рабочий скрипт живёт в `examples/alt-linux/check-updates.sh`.  
Пользователь копирует его в `~/.config/ashell/check-updates.sh`.

Парсер ashell ожидает ровно 4 поля: `pkg old_ver -> new_ver`.  
`awk`-скрипт вырезает из версии epoch-суффикс (`split(v[1], ":")[1]`).

### Зависимости для сборки на ALT Linux

```bash
sudo apt-get install -y pkg-config clang libLLVM-devel \
  libxkbcommon-devel libwayland-devel dbus-devel \
  libpipewire0.3-devel libpulse-devel libudev-devel
```

### Файлы форка

| Файл | Назначение |
|------|-----------|
| `examples/alt-linux/config.toml` | Готовый пример конфига для ALT Linux |
| `examples/alt-linux/check-updates.sh` | Скрипт проверки обновлений через apt-get |
| `docs/src/getting-started/prerequisites.md` | ALT Linux build deps |
| `docs/src/reference/config-reference.md` | ALT Linux пример для Updates модуля |

### sudoers для беспарольного apt-get

```
your_user ALL=(ALL) NOPASSWD: /usr/bin/apt-get
```
