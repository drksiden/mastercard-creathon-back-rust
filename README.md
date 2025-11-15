# Payment Analytics Backend (Rust)

Backend система для аналитики платежей с использованием LLM для генерации SQL запросов из естественного языка.

## 🏗️ Архитектура

- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL с SQLx
- **LLM**: Ollama (локально) или OpenAI (опционально)
- **Logging**: Tracing

## 📋 Требования

- Rust 1.70+
- PostgreSQL 15+
- Ollama (для локального LLM) или OpenAI API ключ

## 🚀 Быстрый старт

### 1. Установка зависимостей

```bash
# Установите Rust (если еще не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Установите PostgreSQL
# На Arch Linux:
sudo pacman -S postgresql

# На Ubuntu/Debian:
sudo apt-get install postgresql postgresql-contrib
```

### 2. Настройка базы данных

```bash
# Запустите PostgreSQL
sudo systemctl start postgresql

# Создайте базу данных
sudo -u postgres psql -c "CREATE DATABASE payment_analytics;"
sudo -u postgres psql -c "CREATE USER postgres WITH PASSWORD 'password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE payment_analytics TO postgres;"
```

### 3. Настройка Ollama (обязательно для работы LLM)

**Подробные инструкции см. в [SETUP.md](SETUP.md)**

Кратко:
```bash
# Установите Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Запустите Ollama (в отдельном терминале или как сервис)
ollama serve

# В другом терминале загрузите модель (выберите одну):
ollama pull llama2        # Легкая (~4GB)
# или
ollama pull mistral       # Баланс (~4GB)
# или
ollama pull mixtral:8x7b-instruct  # Мощная (~26GB)
```

**Важно:** Укажите загруженную модель в `.env` файле в поле `OLLAMA_MODEL`

### 4. Настройка окружения

Создайте файл `.env` в корне проекта:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/payment_analytics
LLM_PROVIDER=ollama
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=mixtral:8x7b-instruct
HOST=0.0.0.0
PORT=3000
RUST_LOG=info,payment_analytics_backend=debug
```

### 5. Запуск

```bash
# Сборка и запуск
cargo run

# Или в режиме разработки с автоперезагрузкой
cargo watch -x run
```

Сервер запустится на `http://localhost:3000`

## 📡 API Endpoints

### Health Check

```bash
GET /api/health
```

Ответ:
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T00:00:00Z",
  "database": "connected",
  "llm": "ollama"
}
```

### Query (Генерация SQL из естественного языка)

```bash
POST /api/query
Content-Type: application/json

{
  "question": "Сколько транзакций было в 2024 году?"
}
```

Ответ:
```json
{
  "question": "Сколько транзакций было в 2024 году?",
  "sql": "SELECT COUNT(*) as total_transactions FROM transactions WHERE transaction_timestamp >= '2024-01-01' AND transaction_timestamp < '2025-01-01';",
  "data": [
    {
      "total_transactions": 2500
    }
  ],
  "execution_time_ms": 45,
  "row_count": 1
}
```

### Примеры вопросов

- "Сколько транзакций было сегодня?"
- "Топ 10 городов по объему транзакций"
- "Средний чек для карт Halyk Bank в Алматы"
- "Объем транзакций по категориям MCC за последний месяц"
- "Транзакции по типам кошельков (Apple Pay, Google Pay)"
- "Дневной объем транзакций за последние 7 дней"

## 🗄️ Структура базы данных

### Таблицы

- **transactions**: Основная таблица транзакций
  - `id`: SERIAL PRIMARY KEY
  - `transaction_id`: Уникальный идентификатор транзакции
  - `transaction_timestamp`: Время транзакции
  - `card_id`: Идентификатор карты
  - `expiry_date`: Срок действия карты (MM/YY)
  - `issuer_bank_name`: Банк-эмитент карты
  - `merchant_id`: Идентификатор мерчанта
  - `merchant_mcc`: Merchant Category Code
  - `mcc_category`: Категория MCC (Retail, Restaurants, Gas Stations, etc.)
  - `merchant_city`: Город мерчанта
  - `transaction_type`: Тип транзакции (Purchase, Refund, Authorization, Reversal)
  - `transaction_amount_kzt`: Сумма транзакции в KZT
  - `original_amount`: Оригинальная сумма (если была конвертация валют)
  - `transaction_currency`: Валюта транзакции (KZT, USD, EUR, etc.)
  - `acquirer_country_iso`: ISO код страны эквайера
  - `pos_entry_mode`: Способ ввода (Chip, Contactless, Magnetic Stripe, etc.)
  - `wallet_type`: Тип кошелька (Apple Pay, Google Pay, Samsung Pay, или NULL)

- **query_audit_log**: Лог всех запросов для аудита
  - `id`: SERIAL PRIMARY KEY
  - `user_id`: Идентификатор пользователя
  - `question`: Вопрос пользователя
  - `generated_sql`: Сгенерированный SQL запрос
  - `success`: Успешность выполнения
  - `error_message`: Сообщение об ошибке (если есть)
  - `execution_time_ms`: Время выполнения в миллисекундах
  - `created_at`: Время создания записи

### Миграции

Миграции автоматически выполняются при запуске приложения из папки `migrations/`. 
При первом запуске создается таблица `transactions` и генерируется 5000 тестовых транзакций.

## 🔧 Разработка

### Структура проекта

```
src/
├── main.rs          # Точка входа
├── config.rs        # Конфигурация
├── error.rs         # Обработка ошибок
├── state.rs         # Состояние приложения
├── db/              # Работа с БД
│   ├── pool.rs
│   ├── queries.rs
│   └── mock_data.rs
├── llm/             # LLM клиент
│   ├── client.rs
│   ├── prompts.rs
│   └── validator.rs
├── api/             # API endpoints
│   ├── health.rs
│   ├── query.rs
│   └── models.rs
└── utils/           # Утилиты
    ├── logger.rs
    └── metrics.rs
```

### Тестирование

```bash
# Запуск тестов
cargo test

# Проверка кода
cargo clippy

# Форматирование
cargo fmt
```

## 🐛 Troubleshooting

### Ошибка подключения к БД

Убедитесь, что:
- PostgreSQL запущен: `sudo systemctl status postgresql`
- База данных создана
- `DATABASE_URL` в `.env` правильный

### Ошибка подключения к Ollama

Убедитесь, что:
- Ollama запущен: `ollama serve`
- Модель загружена: `ollama list`
- `OLLAMA_URL` в `.env` правильный

### Проблемы с миграциями

```bash
# Выполните миграции вручную
psql -U postgres -d payment_analytics -f migrations/001_init.sql
```

## 📝 TODO

- [ ] Добавить аутентификацию
- [ ] Реализовать streaming ответов
- [ ] Добавить кэширование запросов
- [ ] Улучшить валидацию SQL
- [ ] Добавить метрики (Prometheus)
- [ ] Реализовать OpenAI fallback

## 📄 Лицензия

MIT

