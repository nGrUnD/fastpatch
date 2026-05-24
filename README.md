# fastpatch

Графический клиент для [zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) на Windows: установка zapret, подбор стратегий, проверка Discord / YouTube / Cloudflare / EA, управление hosts и настройками — без ручного запуска `.bat` и лишних окон консоли.

Стек: **Tauri 2** · **React 19** · **Rust** · **Vite** · **Tailwind CSS**

Иконка: `app-icon-source.png` (1024×1024) → `pnpm tauri icon app-icon-source.png` → `src-tauri/icons/`.

## Возможности

- **Подключить** на главной — установка zapret (если нужно), автоподбор первой рабочей стратегии (Discord + YouTube, ответ быстрее 3 с), запуск `winws.exe` скрыто.
- **Стратегии** — список `general*.bat`, фильтры по тегам, ручной запуск, тест сервисов, **автоскан** всех стратегий с отменой, добавление своего `.bat`.
- **Проверка связи** — по тегам активной стратегии (Discord, YouTube, Cloudflare CDN, EA/Apex и др.).
- **Hosts** — скачивание и запись блока zapret в системный `hosts` (нужны права администратора).
- **Система** — автозапуск, фильтр игр, ipset, версия zapret, релизы GitHub.
- **Apex Legends** — пресет из issue [#6503](https://github.com/Flowseal/zapret-discord-youtube/issues/6503), проверка EA.
- Один экземпляр **winws.exe**: при скане/тесте текущая стратегия временно отключается и восстанавливается.
- Иконка в трее, сворачивание в трей.

## Требования

| | |
|---|---|
| ОС | Windows 10/11 (x64) |
| Права | **Администратор** (WinDivert / `winws.exe`) |
| Сборка | [Node.js](https://nodejs.org/) 20+, [pnpm](https://pnpm.io/), [Rust](https://rustup.rs/) stable |

Для разработки: PowerShell **от имени администратора** в корне репозитория.

## Быстрый старт (разработка)

```powershell
cd D:\Git\fastpatch
pnpm install
pnpm tauri dev
```

Откроется окно приложения и Vite на `http://localhost:1420/`.  
В debug-сборке UAC не запрашивается повторно — запускайте терминал уже с правами админа.

## Сборка установщика

```powershell
pnpm tauri build
```

Готовый MSI:

`src-tauri\target\release\bundle\msi\fastpatch_*_x64_en-US.msi`

Для обычного использования установите MSI. При первом подключении стратегии нажмите «Запустить от имени администратора» в приложении (один UAC).

**Если приложение сразу закрывается:** установите [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/). Лог ошибок: `%APPDATA%\fastpatch\startup.log`.

**Автозапуск** (Настройки → Приложение): создаётся задача Windows «fastpatch» с высоким приоритетом при входе в систему. UAC показывается **один раз при включении** автозапуска, не при каждой загрузке ПК. Можно включить автоподключение последней стратегии.

## Структура проекта

```
fastpatch/
├── src/                 # React UI (страницы, store, компоненты)
├── src-tauri/           # Rust: команды Tauri, запуск winws, пробы HTTP
│   └── src/commands/    # strategy, probe, hosts, updater, apex, …
├── strategies.json      # Fallback-список стратегий без установленного zapret
├── resources/zapret-extra/  # Бандл: Apex list/bat
└── dist/                # Сборка фронтенда (в .gitignore)
```

Папка **zapret** (с `bin/winws.exe` и `general*.bat`) создаётся при установке рядом с exe или в `target/debug/` при dev — в git не коммитится.

## Теги стратегий

| Тег | Назначение |
|-----|------------|
| Discord | Discord, gateway, CDN |
| YouTube | YouTube и `generate_204` |
| Cloudflare CDN | cloudflare.com |
| Игры | Game filter (UDP — проверка только в игре) |
| Apex Legends | EA / Origin / Apex |
| Общий | Универсальные `general*.bat` |

В автоскане и автоподборе: Discord (gw + updates) + YouTube; задержка **≥ 3000 ms** считается «нет ответа» (красный бейдж в UI).

## Полезные команды

```powershell
pnpm dev          # только фронт (Vite)
pnpm build        # tsc + vite build
pnpm tauri dev    # dev с hot-reload
pnpm tauri build  # release + MSI/NSIS
```

## Ограничения

- Запуск стратегий только на **Windows**.
- Одновременно работает только **один** `winws.exe`.
- Проверка GitHub (релизы zapret) — вручную в настройках; при лимите API ошибка показывается только там, не при старте приложения.

## Связанные проекты

- [Flowseal/zapret-discord-youtube](https://github.com/Flowseal/zapret-discord-youtube) — исходные `.bat`, `service.bat`, списки.
- [bol-van/zapret](https://github.com/bol-van/zapret) — ядро обхода DPI.

## Лицензия

Уточните лицензию репозитория при публикации. Zapret распространяется на условиях своих авторов.
