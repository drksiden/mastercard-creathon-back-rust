# 📖 Руководство по использованию Payment Analytics Backend

## 🎯 Как это работает?

Система преобразует вопросы на естественном языке в SQL-запросы с помощью LLM (Ollama/OpenAI), выполняет их в PostgreSQL и возвращает результаты.

### Процесс работы:

```
1. Пользователь отправляет вопрос → POST /api/query
   {"question": "Сколько транзакций было сегодня?"}

2. Backend отправляет вопрос в LLM (Ollama)
   LLM генерирует SQL: SELECT COUNT(*) FROM transactions WHERE DATE(transaction_timestamp) = CURRENT_DATE;

3. Backend валидирует SQL (проверяет, что это только SELECT)

4. Backend выполняет SQL в PostgreSQL

5. Backend возвращает результат в JSON
   {
     "question": "...",
     "sql": "SELECT ...",
     "data": [...],
     "execution_time_ms": 45
   }
```

## 🚀 Запуск системы

### Шаг 1: Подготовка базы данных

```bash
# Запустите PostgreSQL
sudo systemctl start postgresql

# Создайте базу данных
sudo -u postgres psql -c "CREATE DATABASE payment_analytics;"
```

### Шаг 2: Настройка Ollama (для LLM)

```bash
# Установите Ollama (если еще не установлен)
curl -fsSL https://ollama.com/install.sh | sh

# Запустите Ollama сервер (в отдельном терминале)
ollama serve

# В другом терминале загрузите модель
ollama pull llama2  # или mixtral:8x7b-instruct (больше, но лучше)
```

### Шаг 3: Настройка .env

Создайте файл `.env` в корне проекта:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/payment_analytics
LLM_PROVIDER=ollama
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama2
HOST=0.0.0.0
PORT=3000
RUST_LOG=info,payment_analytics_backend=debug
```

### Шаг 4: Запуск сервера

```bash
# Перейдите в директорию проекта
cd /home/danil/dev/hackaton/mastercard-creathon-back-rust

# Запустите сервер
cargo run
```

Вы увидите:
```
Configuration loaded
Database connected
Migrations completed
LLM client initialized: ollama
Warming up LLM...
LLM ready!
🚀 Server running on http://0.0.0.0:3000
```

## 📡 Использование API

### 1. Health Check

Проверка, что сервер работает:

```bash
curl http://localhost:3000/api/health
```

Ответ:
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z",
  "database": "connected",
  "llm": "ollama"
}
```

### 2. Запрос к базе данных (основной endpoint)

#### Пример 1: Простой подсчет

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question": "Сколько всего транзакций в базе?"}'
```

Ответ:
```json
{
  "question": "Сколько всего транзакций в базе?",
  "sql": "SELECT COUNT(*) as total FROM transactions;",
  "data": [
    {
      "total": 5000
    }
  ],
  "execution_time_ms": 12,
  "row_count": 1
}
```

#### Пример 2: Топ мерчантов

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question": "Топ 5 мерчантов по объему транзакций"}'
```

Ответ:
```json
{
  "question": "Топ 5 мерчантов по объему транзакций",
  "sql": "SELECT merchant_id, SUM(transaction_amount_kzt) as total_volume FROM transactions WHERE transaction_type = 'Purchase' GROUP BY merchant_id ORDER BY total_volume DESC LIMIT 5;",
  "data": [
    {"merchant_id": 123, "total_volume": "125000.50"},
    {"merchant_id": 456, "total_volume": "98000.25"},
    ...
  ],
  "execution_time_ms": 45,
  "row_count": 5
}
```

#### Пример 3: Фильтрация по дате и банку

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question": "Средний чек для карт Halyk Bank в Алматы за последний месяц"}'
```

#### Пример 4: Группировка по категориям

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question": "Объем транзакций по категориям MCC за сегодня"}'
```

## 🌐 Использование из браузера (JavaScript)

### Простой HTML пример

Создайте файл `test.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Payment Analytics Test</title>
</head>
<body>
    <h1>Payment Analytics Query</h1>
    <input type="text" id="question" placeholder="Введите вопрос..." style="width: 400px;">
    <button onclick="sendQuery()">Отправить</button>
    <div id="result"></div>

    <script>
        async function sendQuery() {
            const question = document.getElementById('question').value;
            const resultDiv = document.getElementById('result');
            
            resultDiv.innerHTML = 'Загрузка...';
            
            try {
                const response = await fetch('http://localhost:3000/api/query', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ question })
                });
                
                const data = await response.json();
                
                resultDiv.innerHTML = `
                    <h3>SQL запрос:</h3>
                    <pre>${data.sql}</pre>
                    <h3>Результаты (${data.row_count} строк):</h3>
                    <pre>${JSON.stringify(data.data, null, 2)}</pre>
                    <p>Время выполнения: ${data.execution_time_ms}ms</p>
                `;
            } catch (error) {
                resultDiv.innerHTML = `<p style="color: red;">Ошибка: ${error.message}</p>`;
            }
        }
    </script>
</body>
</html>
```

Откройте в браузере и используйте!

## 🐍 Использование из Python

```python
import requests

# Запрос к API
response = requests.post(
    'http://localhost:3000/api/query',
    json={'question': 'Сколько транзакций было сегодня?'}
)

data = response.json()
print(f"SQL: {data['sql']}")
print(f"Результаты: {data['data']}")
print(f"Время: {data['execution_time_ms']}ms")
```

## 📊 Примеры вопросов, которые можно задавать

### По времени:
- "Сколько транзакций было сегодня?"
- "Транзакции за последние 7 дней"
- "Объем транзакций по месяцам в 2024 году"
- "Транзакции за вчера"

### По мерчантам:
- "Топ 10 мерчантов по количеству транзакций"
- "Транзакции в городе Алматы"
- "Мерчанты категории Retail"

### По банкам:
- "Транзакции по картам Halyk Bank"
- "Средний чек для Kaspi Bank"
- "Сравнение транзакций по банкам"

### По категориям:
- "Объем транзакций по категориям MCC"
- "Топ категории по количеству транзакций"
- "Средний чек в категории Restaurants"

### По типам транзакций:
- "Количество возвратов (Refund)"
- "Соотношение Purchase и Refund"
- "Транзакции типа Authorization"

### По кошелькам:
- "Транзакции через Apple Pay"
- "Сравнение Apple Pay и Google Pay"
- "Доля бесконтактных платежей"

### Комбинированные:
- "Средний чек для карт Halyk Bank в Алматы за последний месяц"
- "Топ 5 городов по объему транзакций через Apple Pay"
- "Дневной объем транзакций в категории Retail за последние 30 дней"

## 🔍 Отладка

### Проверка логов

Сервер выводит логи в консоль:
```
INFO payment_analytics_backend: Received question: Сколько транзакций было сегодня?
INFO payment_analytics_backend: Generated SQL: SELECT COUNT(*) ...
INFO payment_analytics_backend: Query executed successfully: 1 rows in 45ms
```

### Проверка базы данных

```bash
# Подключитесь к базе
psql -U postgres -d payment_analytics

# Проверьте количество транзакций
SELECT COUNT(*) FROM transactions;

# Посмотрите примеры транзакций
SELECT * FROM transactions LIMIT 5;

# Проверьте лог запросов
SELECT * FROM query_audit_log ORDER BY created_at DESC LIMIT 10;
```

### Типичные проблемы

1. **Ошибка подключения к базе данных**
   - Проверьте, что PostgreSQL запущен: `sudo systemctl status postgresql`
   - Проверьте `DATABASE_URL` в `.env`

2. **Ошибка подключения к Ollama**
   - Проверьте, что Ollama запущен: `curl http://localhost:11434/api/tags`
   - Проверьте `OLLAMA_URL` в `.env`
   - Убедитесь, что модель загружена: `ollama list`

3. **LLM генерирует неправильный SQL**
   - Попробуйте другую модель (например, `mixtral:8x7b-instruct`)
   - Уточните вопрос
   - Проверьте логи для просмотра сгенерированного SQL

## 🎯 Следующие шаги

1. **Интеграция с фронтендом** - создайте React/Vue интерфейс
2. **Telegram бот** - добавьте бота для удобного доступа
3. **Кэширование** - кэшируйте часто задаваемые вопросы
4. **Аутентификация** - добавьте авторизацию пользователей
5. **Визуализация** - добавьте графики и диаграммы

