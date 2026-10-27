# Участие в разработке

Используйте Issues для воспроизводимых ошибок и предложений, pull request — для изменений.
Пишите по-русски или по-английски. Обсуждайте крупные изменения механики до реализации.
Не добавляйте в репозиторий личные пути, токены, сохранения партий и настройки IDE.

## Среда

Rust закреплён в `rust-toolchain.toml`; генераторы требуют Python 3.11+ без сторонних пакетов.
На macOS нужны Xcode Command Line Tools, на Windows — инструменты сборки MSVC.
Ubuntu/Debian:

```sh
sudo apt-get install build-essential pkg-config libasound2-dev libudev-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev libvulkan-dev
```

## Проверки

```sh
python3 tools/build_deck.py
python3 tools/build_cards.py
python3 tools/build_audit.py
python3 tools/build_coins.py
python3 tools/validate_assets.py
python3 tools/check_repository.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -- --check-assets
```

Коммитьте результаты генерации вместе с исходниками: CI запрещает расхождения.
Для быстрой проверки одного движка: `cargo test -p orcs-core --locked`.
Снимок интерфейса требует графического сеанса:
`cargo run --locked -- --preview --capture output/table-preview.png`.
Автотесты не заменяют проверку живой партии и физической печати.

## Контент

Источники карт — `tools/build_deck.py` и `tools/content.py`, данные — `data/cards.json`.
Каталог встраивается в бинарник; после его изменения пересоберите игру.
Правила в `rules/quickstart.md` встраиваются в экран правил.
Иллюстрации лежат в `assets/art/`; журнал промптов — в `prompts/generated.json`.
Записывайте только проектные пути. Генераторы не создают PNG автоматически.

Сохраняйте согласованность кода, правил, каталога и печатных карточек.
Для изменения механики добавляйте сценарий, проверяющий поведение и сохранность карт.
Не переносите новые функции в справочный Godot-прототип.

## Pull request

Опишите проблему, итоговое поведение и выполненные проверки.
Для интерфейса приложите снимок, для механики — пример хода или партии.
Вклад распространяется под лицензией MIT проекта.
