# stevedore

[English](./README.md) · **Русский**

[![CI](https://github.com/cclxxi/stevedore/actions/workflows/ci.yml/badge.svg)](https://github.com/cclxxi/stevedore/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/stevedore.svg)](https://crates.io/crates/stevedore)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)

Маленький консольный TUI для наблюдения за Docker-контейнерами и их логами —
крошечная read-only альтернатива Portainer, которая живёт прямо в терминале.

Никакого веб-сервера, агента или базы. Просто один бинарник **~1 МБ**, который
общается с локальным Docker-сокетом. Сделано, чтобы быстро глянуть «а что там с
контейнерами» — особенно по SSH на сервере.

<!-- Абсолютный URL, чтобы демо рендерилось и на GitHub, и на crates.io.
     Перезаписать: vhs assets/demo.tape -->
![демо](https://raw.githubusercontent.com/cclxxi/stevedore/master/assets/demo.gif)

## Почему stevedore

- **Лёгкий** — один оптимизированный бинарник ~1 МБ, почти нулевой расход в простое.
- **Без настройки** — не нужен демон, агент или конфиг. Запусти там, где крутятся контейнеры.
- **Read-only и безопасный** — только смотрит, ничего (пока) не запускает и не останавливает.
- **Удобно по SSH** — то что надо, чтобы быстро проверить контейнеры на удалённом сервере.

## Возможности

- **Список контейнеров** с живым статусом, образом, портами и состоянием
- **Живые метрики** — CPU %, память (использование/лимит), сетевой I/O по каждому контейнеру
- **Просмотр логов** — follow в реальном времени, прокрутка назад, прыжки в начало/конец
- Переключение **только запущенные / все**
- Аккуратная обработка ошибок, если демон недоступен (без паник, терминал всегда восстанавливается)

## Требования

- Доступный Docker-демон (по умолчанию сокет `/var/run/docker.sock`)
- У пользователя есть доступ к сокету (группа `docker` или запуск через `sudo`)
- Rust 1.88+ (только если собираешь из исходников)

## Установка

### Homebrew (macOS / Linux)

```sh
brew install cclxxi/tap/stevedore
```

### Скрипт установки (Rust не нужен)

Поставит последний релизный бинарник под твою платформу:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cclxxi/stevedore/releases/latest/download/stevedore-installer.sh | sh
```

### Через crates.io

```sh
cargo install stevedore
```

### Готовые бинарники

Скачай архив под свою платформу со страницы
[Releases](https://github.com/cclxxi/stevedore/releases).

### Из исходников

```sh
git clone https://github.com/cclxxi/stevedore
cd stevedore
cargo install --path .
```

## Запуск

```sh
stevedore
```

### Горячие клавиши

| Клавиша | Действие |
|---|---|
| `↑`/`k`, `↓`/`j` | Перемещение по списку / прокрутка логов |
| `g` / `G` | В начало / в конец |
| `a` | Переключить все / только запущенные |
| `Enter` / `l` | Открыть логи выбранного контейнера |
| `PgUp` / `PgDn` | Постраничная прокрутка логов |
| `f` | Включить/выключить follow логов |
| `Esc` / `q` | Назад к списку (в логах) / выход (в списке) |
| `?` | Показать справку |
| `q` | Выход |

## Контрибьют

Буду рад любым правкам! В [CONTRIBUTING.md](./CONTRIBUTING.md) описано, как
поднять окружение, прогнать проверки и прислать PR. Задачи для старта помечены
лейблом [`good first issue`](https://github.com/cclxxi/stevedore/labels/good%20first%20issue)
— отличная точка входа.

## Лицензия

Распространяется под [GNU General Public License v3.0 или новее](./LICENSE).

© 2026 Ilia Proshin.
