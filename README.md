# Zmk battery client

Zmk client connects to specified device and produces json output usable in custom waybar module.
Application retrieves battery levels of all parts of keyboard(both halves in case of split keeb).
The output text is battery percentage of keyboard half with lower value.
The tooltip contains battery levels of all  keyboard parts.

## Usage

Put custom zmk module in waybar:

```

  "custom/zmk": {
    "format": "   {}",
    "tooltip": true,
    "interval": 300,
    "exec": "ZmkBatteryClient E1:E2:69:AB:CE:90",
    "return-type": "json"
  },
```

## Plan

- [ ] By default produce general json output usable for another applications
- [ ] Produce waybar json output only if flag specified(`--waybar`)

===========================================

# Пошаговая инструкция по установке и настройке ZmkBatteryClient в Linux Mint

Ниже приведена единая практическая инструкция для начинающего пользователя по установке, запуску и использованию `ZmkBatteryClient` в Linux Mint, а также по настройке отображения заряда split-клавиатуры без Waybar — через терминал и Conky.

## 1. Назначение программы

`ZmkBatteryClient` — это утилита на Rust, которая подключается к Bluetooth-устройству ZMK и выводит информацию о заряде в формате JSON. Такой формат удобен для панелей типа Waybar, но может использоваться и отдельно в терминале или в Conky. В ходе установки на данной системе программа уже была успешно собрана, а команда запуска вернула корректный результат с зарядом клавиатуры. 

Типовой запуск выполняется так:

```bash
cd ~/Загрузки/ZmkBatteryClient-master
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
```

## 2. Почему Waybar не подходит для Linux Mint

В текущем окружении используется сеанс `X11` и рабочий стол `Cinnamon`. Для такого режима Waybar использовать не следует.

Причина состоит в том, что Waybar позиционируется как панель для Wayland-окружений, прежде всего для Sway и wlroots-совместимых композиторов. Дополнительно библиотека `gtk-layer-shell`, на которую опираются подобные панели, прямо указывает, что работает только в Wayland и не поддерживается на любых X11-рабочих столах. Следовательно, пакет `waybar` можно установить, но в `X11 + Cinnamon` он не является штатным и совместимым решением для отображения панели. 

Именно поэтому для Linux Mint Cinnamon рекомендуется не Waybar, а один из следующих вариантов:

* запуск из терминала;
* вывод в Conky;
* отдельный собственный индикатор или скрипт.

## 3. Подготовка системы

Для сборки проекта под Linux Mint требуются:

* `bluez`;
* `libdbus-1-dev`;
* `pkg-config`;
* `build-essential`;
* `unzip`;
* `curl`.

Библиотека `bluer`, используемая проектом, требует работающий `bluetoothd`, а для сборки на Debian/Ubuntu нужны D-Bus headers из пакета `libdbus-1-dev`. ([docs.rs][2])

Установка выполняется так:

```bash
sudo apt update
sudo apt install -y bluez libdbus-1-dev pkg-config build-essential unzip curl
```

После этого следует убедиться, что Bluetooth-служба запущена:

```bash
systemctl status bluetooth
```

Если служба не активна, требуется выполнить:

```bash
sudo systemctl enable --now bluetooth
```

## 4. Установка Rust

Проект использует Rust 2024 edition, а поддержка этой редакции стабилизирована в Rust 1.85.0. Поэтому рекомендуется устанавливать Rust через `rustup`, а не через устаревший пакет дистрибутива. 

Команда установки:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

После установки требуется подключить окружение Rust к текущему терминалу:

```bash
source "$HOME/.cargo/env"
```

Проверка:

```bash
rustc --version
cargo --version
```

В выполненной установке были получены `rustc 1.95.0` и `cargo 1.95.0`, то есть версия подходит для сборки данного проекта. 

## 5. Распаковка и сборка проекта

Если архив `ZmkBatteryClient-master.zip` находится в каталоге `~/Загрузки`, выполняется следующая последовательность:

```bash
cd ~/Загрузки
unzip ZmkBatteryClient-master.zip
cd ZmkBatteryClient-master
cargo build --release
```

В уже выполненном журнале установки сборка завершилась успешно, после чего был создан исполняемый файл в каталоге `target/release/`. 

## 6. Первый запуск

Запуск выполняется так:

```bash
cd ~/Загрузки/ZmkBatteryClient-master
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
```

Примеры возможного вывода:

```json
{"text":"69%","tooltip":"Central: 69%","class":"zmk_battery"}
```

или, после корректной настройки прошивки split-клавиатуры:

```json
{"text":"68%","tooltip":"Central: 68%\nPeripheral 0: 79%","class":"zmk_battery"}
```

Если в выводе виден `\n`, это нормально: программа печатает JSON, а в JSON перевод строки внутри строки кодируется как `\n`. Такой формат как раз ожидается пользовательскими модулями Waybar при `return-type: "json"`. 

## 7. Почему сначала отображалась только одна половина

Если вывод был вида:

```json
{"text":"68%","tooltip":"Central: 68%","class":"zmk_battery"}
```

это означало, что хост видел только заряд центральной половины.

Для split-клавиатур ZMK по умолчанию передаётся только заряд central side. Для отображения заряда периферийной половины необходимо отдельно включить параметр получения уровня заряда периферии и его проксирования на центральную сторону. Документация ZMK прямо указывает, что для отображения обеих половин должны быть включены обе опции:

* `CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_FETCHING=y`
* `CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_PROXY=y` 

## 8. Как доработать прошивку для отображения второй половины

В конфигурационный `.conf` файл прошивки ZMK, который участвует в сборке центральной половины, нужно добавить следующие строки:

```conf
CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_FETCHING=y
CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_PROXY=y
```

Где именно это добавлять:

* в общий `.conf` проекта, если обе половины собираются с ним;
* либо в `.conf` центральной половины;
* либо в другой активный Kconfig-файл прошивки, где уже задаются `CONFIG_ZMK_*`.

После добавления параметров требуется:

1. пересобрать прошивку ZMK;
2. перепрошить клавиатуру;
3. заново подключить её по Bluetooth;
4. снова выполнить:

```bash
cd ~/Загрузки/ZmkBatteryClient-master
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
```

Если прошивка настроена правильно, в `tooltip` появятся данные по обеим половинам, например:

```json
{"text":"68%","tooltip":"Central: 68%\nPeripheral 0: 79%","class":"zmk_battery"}
```

Это будет означать, что центральная половина действительно получает и отдаёт хосту заряд периферийной половины. 

## 9. Использование без Waybar через терминал

### 9.1. Простой запуск

Для обычной проверки достаточно выполнить:

```bash
cd ~/Загрузки/ZmkBatteryClient-master
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
```

### 9.2. Более удобный вывод в терминале

Так как программа выводит JSON, для «человеческого» вида удобно использовать `python3`, который уже имеется в Linux Mint:

```bash
cd ~/Загрузки/ZmkBatteryClient-master
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53 | python3 -c 'import sys, json; d=json.load(sys.stdin); print("Battery:", d["text"]); print(d["tooltip"])'
```

Пример результата:

```text
Battery: 68%
Central: 68%
Peripheral 0: 79%
```

Такой способ не требует изменения исходного кода программы и подходит для повседневной проверки.

## 10. Установка бинарного файла в пользовательский путь

Чтобы не запускать программу каждый раз из каталога проекта, можно установить её в `~/.local/bin`:

```bash
mkdir -p ~/.local/bin
install -m 755 ~/Загрузки/ZmkBatteryClient-master/target/release/ZmkBatteryClient ~/.local/bin/ZmkBatteryClient
```

Проверка пути:

```bash
echo $PATH
which ZmkBatteryClient
```

Если `~/.local/bin` уже присутствует в `PATH`, программу можно запускать так:

```bash
ZmkBatteryClient DD:5E:72:DC:9B:53
```

Однако для данной инструкции базовым примером остаётся именно команда:

```bash
./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
```

## 11. Использование через Conky

Conky в данном окружении является основным подходящим способом постоянного отображения заряда на рабочем столе X11/Cinnamon.

### 11.1. Как найти конфигурацию Conky

По умолчанию Conky использует пользовательскую конфигурацию:

```bash
~/.config/conky/conky.conf
```

Системный образец обычно находится в:

```bash
/etc/conky/conky.conf
```

Официальная документация Conky указывает эти пути как стандартные. Она также позволяет вывести встроенную конфигурацию через `conky -C`, запускать Conky с явным файлом через `-c` и отправлять его в фон через `-d`. 

Если нужно узнать, какой файл используется в данный момент:

```bash
ps -ef | grep '[c]onky'
```

Если пользовательского файла нет, можно создать его из встроенного шаблона:

```bash
mkdir -p ~/.config/conky
conky -C > ~/.config/conky/conky.conf
```

### 11.2. Куда вставлять строки

В текущем приложенном `conky.conf` уже есть блок:

```lua
conky.text = [[
...
]]
```

Именно в этот блок, непосредственно перед последней строкой `]]`, нужно добавлять строки отображения. В текущем файле уже заданы параметры `font = 'DejaVu Sans Mono:size=12'` и `gap_y = 60`, из-за чего виджет получается довольно крупным и длинным. 

### 11.3. Какие строки добавить

Если Conky запускается не из каталога проекта, относительный путь `./target/release/...` работать не будет. Поэтому для Conky рекомендуется использовать полный путь к бинарному файлу.

Пример для текущего расположения проекта:

```lua
${color grey}ZMK keyboard:$color ${execi 300 /home/ex1/Загрузки/ZmkBatteryClient-master/target/release/ZmkBatteryClient DD:5E:72:DC:9B:53 | sed -n 's/.*"text":"\([^"]*\)".*/\1/p'}
${color grey}Battery details:$color ${execi 300 /home/ex1/Загрузки/ZmkBatteryClient-master/target/release/ZmkBatteryClient DD:5E:72:DC:9B:53 | python3 -c 'import sys, json; d=json.load(sys.stdin); print(d["tooltip"].replace("\n", " | "))'}
```

Эти строки нужно вставить прямо перед `]]`.

`execi` является штатным механизмом Conky для периодического запуска внешней команды из секции `conky.text`. Документация Conky указывает, что переменные отображения размещаются именно в `conky.text`, а глобальные параметры — в `conky.config`. 

### 11.4. Что получится на экране

После запуска Conky должны появиться строки примерно такого вида:

```text
ZMK keyboard: 68%
Battery details: Central: 68% | Peripheral 0: 79%
```

Если требуется именно вертикальное отображение в две строки, вторую строку можно заменить на более простой вариант без `replace("\n", " | ")`, но тогда потребуется более аккуратная обработка вывода.

## 12. Как уменьшить масштаб Conky, если строки уходят за экран

Если строки уходят ниже границы экрана, следует уменьшить шрифт и уменьшить вертикальный отступ. В текущем файле используются:

```lua
font = 'DejaVu Sans Mono:size=12',
gap_y = 60,
```

Их можно заменить, например, на:

```lua
font = 'DejaVu Sans Mono:size=9',
gap_y = 20,
```

Такие параметры уменьшают общий размер текста и поднимают окно выше. В вашем текущем `conky.conf` действительно используется размер шрифта `12` и `gap_y = 60`. 

## 13. Как перезапустить Conky

Если требуется перечитать изменённую конфигурацию, выполняется:

```bash
killall -SIGUSR1 conky
```

Если нужен полный перезапуск:

```bash
killall conky
conky -c ~/.config/conky/conky.conf -d
```

Официальная man-страница Conky прямо указывает:

* `-c` — загрузка указанного файла конфигурации;
* `-d` — запуск в фоне;
* `killall -SIGUSR1 conky` — перечитывание `~/.config/conky/conky.conf` без полного завершения процесса. ([Ubuntu Manpages][8])

## 14. Итоговый порядок действий

1. Установить системные зависимости:

   ```bash
   sudo apt update
   sudo apt install -y bluez libdbus-1-dev pkg-config build-essential unzip curl
   ```

2. Проверить Bluetooth:

   ```bash
   systemctl status bluetooth
   ```

3. Установить Rust:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

4. Распаковать архив и собрать проект:

   ```bash
   cd ~/Загрузки
   unzip ZmkBatteryClient-master.zip
   cd ZmkBatteryClient-master
   cargo build --release
   ```

5. Выполнить тестовый запуск:

   ```bash
   ./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53
   ```

6. Если виден только `Central`, доработать прошивку ZMK:

   ```conf
   CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_FETCHING=y
   CONFIG_ZMK_SPLIT_BLE_CENTRAL_BATTERY_LEVEL_PROXY=y
   ```

7. Пересобрать и перепрошить клавиатуру.

8. Для повседневного использования:

   * либо запускать программу вручную из терминала;
   * либо добавить строки в `~/.config/conky/conky.conf` и использовать Conky.

## 15. Краткий вывод

В данном Linux Mint с `X11` и `Cinnamon` Waybar не является совместимым рабочим вариантом, поскольку он рассчитан на Wayland-окружения, а используемый layer-shell не поддерживается на X11. 

Правильная практическая схема для текущего окружения следующая:

* установка и запуск `ZmkBatteryClient`;
* включение в прошивке ZMK проксирования батареи периферийной половины;
* использование команды
  `./target/release/ZmkBatteryClient DD:5E:72:DC:9B:53`
  для проверки из терминала;
* вывод результата в Conky для постоянного отображения на рабочем столе.

Если требуется, следующей редакцией может быть подготовлен уже полностью готовый `conky.conf` целиком, с внесёнными строками `ZmkBatteryClient` и уменьшенным масштабом отображения.



