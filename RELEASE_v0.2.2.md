## fastpatch v0.2.2

Обновление пресета **Apex Legends**: одна рабочая стратегия **ALT11 APEX**, исправлен возврат в лобби после матча.

### Новое и изменено

- **ALT11 APEX** — единственный пресет Apex (удалён устаревший `general (APEX).bat`)
- Игровой UDP без `ipset-all`, исключения подсетей EA/Respawn (`155.133.0.0/16` и др.)
- Списки `list-apex-extra.txt`, `ipset-exclude-apex-ea.txt`
- Панель Apex: «Подключить ALT11 APEX», установка пресета убирает старый `.bat` из zapret
- При установке пресета игровой фильтр zapret выключается (рекомендация для Apex)

### Установка / обновление с 0.2.1

Скачайте **`fastpatch_0.2.2_x64_en-US.msi`** или **`fastpatch_0.2.2_x64-setup.exe`**.

После обновления: **Apex → «Установить пресет»** → подключите **ALT11 APEX**. Если раньше была стратегия «APEX» — выберите ALT11 APEX один раз.

### Требования

- Windows 10/11 x64
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) при проблемах с окном

Подробнее — [README](https://github.com/nGrUnD/fastpatch/blob/main/README.md).
