# ALT Linux plugins for ashell

Готовые скрипты-плагины для использования с ashell на ALT Linux.

## Установка

```bash
mkdir -p ~/.config/ashell
cp examples/alt-linux/plugins/*.sh ~/.config/ashell/
chmod +x ~/.config/ashell/*.sh
cp examples/alt-linux/config.toml ~/.config/ashell/config.toml
```

---

## Плагины

### `musicplayer.sh` — медиаплеер

Отображает текущий трек через MPRIS/playerctl. Два режима:

| Аргумент | Используется для | Описание |
|----------|-----------------|----------|
| `info` | `MusicPlayer` | Иконка (▶/⏸) + исполнитель — название (до 45 символов) |
| `check` | `MusicPrev`, `MusicNext` | Показывает кнопки только когда плеер активен |

**Зависимости:** `playerctl`

```bash
sudo apt-get install playerctl
```

---

### `vitals.sh` — температуры и мощность (AMD)

Оптимизирован для **Minisforum UM780 XTX (Ryzen 7 7840HS / Radeon 780M)**.  
Показывает: `CPU 45C GPU 38C SSD 32C | 15W | 25.1.0`

| Поле | Источник |
|------|---------|
| CPU temp | `sensors k10temp-pci-00c3` → Tctl |
| GPU temp | `sensors amdgpu-pci-c500` → edge |
| SSD temp | `sensors nvme-pci-0100` → Composite |
| GPU power | `sensors amdgpu-pci-c500` → PPT |
| Mesa version | `glxinfo` |

**Зависимости:** `lm_sensors`, `glxinfo` (`mesa-demos`)

```bash
sudo apt-get install lm_sensors mesa-demos
sudo sensors-detect --auto
```

> ⚠️ Адреса PCI-шины (`pci-00c3`, `pci-c500`) могут отличаться на других машинах.
> Уточнить своим: `sensors | grep -E "k10temp|amdgpu|nvme"`

---

### `battery.sh` — батарея

Универсальный скрипт для ноутбуков через `/sys/class/power_supply`.  
Формат: `5.02W / 95% / 10h 1min`

Состояния `alt`: `charging`, `discharging`, `full`, `unknown`

> Для UM780 XTX (настольный Mini-PC) батарея не актуальна.

---

### `check-updates.sh` — проверка обновлений

Парсит `apt-get dist-upgrade --simulate` и выдаёт список в формате,
который понимает встроенный модуль Updates:

```
<пакет> <старая_версия> -> <новая_версия>
```

**Требует sudo без пароля для apt-get:**

```bash
sudo visudo
# Добавить строку:
your_user ALL=(ALL) NOPASSWD: /usr/bin/apt-get
```

---

## Конфигурация в config.toml

```toml
[[CustomModule]]
name       = "MusicPlayer"
command    = "playerctl play-pause"
listen_cmd = "~/.config/ashell/musicplayer.sh info"

[[CustomModule]]
name       = "Vitals"
command    = "ptyxis -e btop"
listen_cmd = "~/.config/ashell/vitals.sh"

[updates]
interval   = 3600
check_cmd  = "~/.config/ashell/check-updates.sh"
update_cmd = "kitty --hold -e sudo apt-get dist-upgrade -y"
```

Полный пример: [`config.toml`](config.toml)
