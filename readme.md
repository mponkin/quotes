# Проект модуля 2. Стриминг котировок

Проект содержит 3 крейта:
* quotes_lib - библиотека с общим кодом
* quotes_server - сервер котировок
* quotes_lib - клиент котировок

## Запуск

### Сервер

```bash
cargo run --bin quotes_server
```

| Параметр | Описание | Значение по умолчанию|
|-|-|-|
| `--port <PORT>` | задает номер порта для прослушивания | `3000` |
| `--tickers <TICKERS>` | путь к файлу со списком тикеров | `all_tickers.txt` |

### Клиент

```
cargo run --bin quotes_client <SERVER_ADDRESS>:<SERVER_PORT> --port <LOCAL_PORT> --tickers <TICKERS_PATH>
```

| Параметр | Описание | Значение по умолчанию|
|-|-|-|
|<SERVER_ADDRESS>| IP адрес сервера |Обязательный|
|<SERVER_PORT>| TCP порт сервера |Обязательный|
|--port <LOCAL_PORT>| Локальный UDP порт для котировок | Обязательный |
|--tickers <TICKERS_PATH>| Путь к файлу котировок| Обязательный|

### Примечание

Для удобства в корне репозитория есть файлы
* all_tickers.txt - все тикеры
* five_tickers.txt - 5 тикеров
* only_aapl.txt - только AAPL