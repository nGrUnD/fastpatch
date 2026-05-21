## fastpatch v0.1.0

Первый публичный релиз — GUI-обёртка над [zapret-discord-youtube](https://github.com/bol-van/zapret-discord-youtube) для Windows.

### Установка

Скачайте **`fastpatch_0.1.0_x64_en-US.msi`** (рекомендуется) или **`fastpatch_0.1.0_x64-setup.exe`**. Запускайте от имени администратора.

### Возможности

- Подключение одной кнопкой, список стратегий с тегами (Discord, YouTube, Cloudflare CDN и др.)
- Автоскан всех стратегий, ручной тест, добавление своей стратегии из `.bat`
- Проверка доступности по целям стратегии (порог «медленно» — 3000 ms)
- Скрытый запуск `winws`, пауза активной сессии при скане/тесте
- Настройки: zapret, обновления, ссылки на репозиторий fastpatch и релизы

### Требования

- Windows 10/11 x64
- Права администратора (для DPI и winws)

### Сборка из исходников

```bash
pnpm install
pnpm tauri build
```

Подробнее — [README](https://github.com/nGrUnD/fastpatch/blob/main/README.md).
