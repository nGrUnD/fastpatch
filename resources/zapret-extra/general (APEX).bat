@echo off
chcp 65001 > nul
:: fastpatch: Apex Legends preset (zapret-discord-youtube issue #6503, UPD 3)
:: UDP 10000-10100 + ipset-exclude — критично для лобби/матчмейкинга

cd /d "%~dp0"
call service.bat status_zapret
call service.bat check_updates
call service.bat load_game_filter
echo:

set "BIN=%~dp0bin\"
set "LISTS=%~dp0lists\"
cd /d %BIN%

start "zapret: %~n0" /min "%BIN%winws.exe" ^
--ipset="%LISTS%ipset-all.txt" ^
--hostlist="%LISTS%list-apex.txt" ^
--hostlist="%LISTS%list-general.txt" ^
--wf-tcp=80,443,1024-1124,2053,2083,2087,2096,3216,8443,9960-9969,18000,18060,18120,27900,28910,29900,%GameFilterTCP% ^
--wf-udp=443,1024-1124,1400,3478-3481,5349,18000,19294-19344,29900,37000-40000,50000-50100,10000-10100,%GameFilterUDP% ^
--filter-tcp=80 --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake,split2 --dpi-desync-autottl=2 --dpi-desync-fooling=badseq --dpi-desync-badseq-increment=2 --new ^
--filter-tcp=443 --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake --dpi-desync-fake-tls-mod=none --dpi-desync-repeats=6 --dpi-desync-fooling=badseq,badsum --dpi-desync-badseq-increment=1000 --dpi-desync-fake-tls="%BIN%tls_clienthello_max_ru.bin" --new ^
--filter-udp=443 --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake --dpi-desync-repeats=11 --dpi-desync-fake-quic="%BIN%quic_initial_www_google_com.bin" --new ^
--filter-tcp=2053,2083,2087,2096,8443 --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake --dpi-desync-fake-tls-mod=none --dpi-desync-repeats=6 --dpi-desync-fooling=badseq --dpi-desync-badseq-increment=2 --dpi-desync-fake-tls="%BIN%tls_clienthello_max_ru.bin" --new ^
--filter-udp=1400,3478-3481,5349,19294-19344,50000-50100 --ipset-exclude="%LISTS%ipset-exclude.txt" --filter-l7=discord,stun --dpi-desync=fake --new ^
--filter-tcp=%GameFilterTCP% --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake --dpi-desync-fake-tls-mod=none --dpi-desync-repeats=6 --dpi-desync-fooling=badseq,badsum --dpi-desync-badseq-increment=1000 --dpi-desync-fake-tls="%BIN%tls_clienthello_max_ru.bin" --new ^
--filter-udp=%GameFilterUDP% --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync=fake --dpi-desync-autottl=2 --dpi-desync-repeats=10 --dpi-desync-any-protocol=1 --dpi-desync-fake-unknown-udp="%BIN%quic_initial_www_google_com.bin" --dpi-desync-cutoff=n2
