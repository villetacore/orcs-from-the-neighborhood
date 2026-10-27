# Установка

Скачайте ZIP из https://github.com/villetacore/orcs-from-the-neighborhood/releases
и распакуйте целиком. Не переносите один бинарник без ресурсов.

## Windows x64

Откройте `orcs-from-the-neighborhood.exe`. Нужна Windows 10/11 x64 и актуальный
видеодрайвер с поддержкой Vulkan или DirectX 12. Сборки не подписаны сертификатом
издателя, поэтому Windows может показать предупреждение о неизвестном приложении.

## macOS

Для Apple Silicon выбирайте `macos-aarch64`, для Intel — `macos-x86_64`.
Нужна macOS 12+ с поддержкой Metal. Откройте `Orcs from the Neighborhood.app`;
приложение можно перенести в Applications. Сборки не имеют подписи Developer ID
и нотариализации Apple. Если система блокирует запуск, после попытки открытия
разрешите приложение в настройках «Конфиденциальность и безопасность», только
если архив получен из официального репозитория.

## Linux x64

Сборка рассчитана на Ubuntu 24.04+ и совместимые дистрибутивы с glibc 2.39+.
Нужен графический сеанс X11 или Wayland и видеодрайвер с Vulkan.
На Ubuntu при необходимости установите библиотеки:

```sh
sudo apt-get install libx11-6 libxi6 libxrandr2 libxcursor1 libxinerama1 libxkbcommon0 libwayland-client0 libudev1 libvulkan1 fonts-dejavu-core
chmod +x orcs-from-the-neighborhood
./orcs-from-the-neighborhood
```

Пустые квадраты вместо кириллицы: установите `fonts-dejavu-core`.
Ошибка инициализации графики: обновите видеодрайвер и проверьте поддержку Vulkan.

## Проверка загрузки

Скачайте `SHA256SUMS` рядом с архивом и сравните SHA-256 нужного файла.

```sh
# Linux: проверка скачанных архивов
sha256sum --check --ignore-missing SHA256SUMS
# macOS: вычисление суммы одного файла
shasum -a 256 orcs-from-the-neighborhood-0.4.0-macos-aarch64.zip
```

В PowerShell: `Get-FileHash .\имя-архива.zip -Algorithm SHA256`.

## Сохранения и данные

Автосохранение `table_save.json` хранится в папке данных приложения:

- macOS: `~/Library/Application Support/com.villetacore.orcs-from-the-neighborhood/`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/orcs-from-the-neighborhood/`
- Windows: `%APPDATA%\villetacore\orcs-from-the-neighborhood\data\`

Формат сохранения — 4. Более старые несовместимые сохранения отклоняются;
неудачная загрузка не перезаписывает существующий файл. Перед обновлением
можно скопировать его в резервную папку. Параметр `--save file.json` меняет путь.

В игре нет аккаунтов, сетевого режима и аналитики приложения. Исходники движка
и клиента доступны в репозитории. Перед публикацией отчёта об ошибке убирайте
из логов и сохранений личные пути и сведения об игроках.

## Диагностика без графического окна

Запустите бинарник с `--version`, `--help` или `--check-assets`.
В macOS бинарник находится в `Orcs from the Neighborhood.app/Contents/MacOS/`.
`--check-assets` проверяет наличие иллюстраций, но не запускает GPU и не проверяет
интерактивную партию.
