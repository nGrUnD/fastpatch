## fastpatch v0.2.1

Исправление: приложение сразу закрывалось после установки MSI у других пользователей.

### Исправлено

- Иконка трея искалась по пути машины разработчика (`CARGO_MANIFEST_DIR`) — на чужих ПК файл не находился, `setup` падал и окно не открывалось
- Иконка и ресурсы Apex теперь в bundle MSI (`icon.ico`, `strategies.json`, пресеты)
- Убран принудительный UAC при старте (отмена UAC выглядела как «вылет»); права админа — через кнопку в интерфейсе
- Лог ошибок старта: `%APPDATA%\fastpatch\startup.log`

### Установка

`fastpatch_0.2.1_x64_en-US.msi` или `fastpatch_0.2.1_x64-setup.exe`

Если не открывается — установите [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).
