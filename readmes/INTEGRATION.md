# 🔌 Интеграция и использование системы

## 🎯 Что это такое?

Это **полноценная рабочая система** для аналитики платежей, которая:
- ✅ Принимает вопросы на естественном языке (русский/английский)
- ✅ Генерирует SQL запросы через LLM (Gemini/Ollama)
- ✅ Выполняет запросы в PostgreSQL
- ✅ Возвращает результаты в JSON формате
- ✅ Логирует все запросы для аудита

**Это не тесты** - это production-ready backend API, который можно интегрировать с любым фронтендом или сервисом.

## 🚀 Варианты использования

### 1. Веб-интерфейс (React/Vue/Angular)

Создайте фронтенд, который отправляет запросы к вашему API:

```javascript
// React пример
import React, { useState } from 'react';

function AnalyticsDashboard() {
  const [question, setQuestion] = useState('');
  const [result, setResult] = useState(null);
  const [loading, setLoading] = useState(false);

  const handleQuery = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:3000/api/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ question }),
      });
      
      const data = await response.json();
      setResult(data);
    } catch (error) {
      console.error('Error:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <input
        type="text"
        value={question}
        onChange={(e) => setQuestion(e.target.value)}
        placeholder="Задайте вопрос о транзакциях..."
      />
      <button onClick={handleQuery} disabled={loading}>
        {loading ? 'Загрузка...' : 'Отправить'}
      </button>
      
      {result && (
        <div>
          <h3>SQL запрос:</h3>
          <pre>{result.sql}</pre>
          <h3>Результаты ({result.row_count} строк):</h3>
          <pre>{JSON.stringify(result.data, null, 2)}</pre>
          <p>Время выполнения: {result.execution_time_ms}ms</p>
        </div>
      )}
    </div>
  );
}
```

### 2. Telegram бот

Создайте бота на Python, который использует ваш API:

```python
import requests
from telegram import Update
from telegram.ext import Application, CommandHandler, MessageHandler, filters

API_URL = "http://localhost:3000/api/query"

async def handle_message(update: Update, context):
    question = update.message.text
    
    try:
        response = requests.post(
            API_URL,
            json={"question": question},
            timeout=30
        )
        response.raise_for_status()
        data = response.json()
        
        # Форматируем ответ
        result_text = f"📊 Результат:\n\n"
        result_text += f"SQL: `{data['sql']}`\n\n"
        result_text += f"Найдено записей: {data['row_count']}\n"
        result_text += f"Время выполнения: {data['execution_time_ms']}ms\n\n"
        
        # Показываем первые 10 строк
        if data['data']:
            result_text += "Данные:\n"
            for i, row in enumerate(data['data'][:10], 1):
                result_text += f"{i}. {row}\n"
        
        await update.message.reply_text(result_text, parse_mode='Markdown')
    except Exception as e:
        await update.message.reply_text(f"❌ Ошибка: {e}")

def main():
    app = Application.builder().token("YOUR_BOT_TOKEN").build()
    app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_message))
    app.run_polling()

if __name__ == '__main__':
    main()
```

### 3. Python скрипт для автоматизации

```python
import requests
import json
from datetime import datetime

class PaymentAnalytics:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
    
    def query(self, question):
        """Отправить вопрос к API"""
        response = requests.post(
            f"{self.base_url}/api/query",
            json={"question": question},
            timeout=30
        )
        response.raise_for_status()
        return response.json()
    
    def get_total_transactions(self):
        """Получить общее количество транзакций"""
        result = self.query("Сколько всего транзакций в базе?")
        return result['data'][0]['total_transactions']
    
    def get_top_merchants(self, limit=10):
        """Получить топ мерчантов"""
        result = self.query(f"Топ {limit} мерчантов по объему транзакций")
        return result['data']
    
    def get_daily_stats(self, days=7):
        """Получить статистику за последние N дней"""
        result = self.query(
            f"Дневной объем транзакций за последние {days} дней"
        )
        return result['data']

# Использование
analytics = PaymentAnalytics()

# Общее количество транзакций
total = analytics.get_total_transactions()
print(f"Всего транзакций: {total:,}")

# Топ мерчантов
top_merchants = analytics.get_top_merchants(5)
print("\nТоп 5 мерчантов:")
for merchant in top_merchants:
    print(f"  {merchant}")

# Статистика за неделю
weekly_stats = analytics.get_daily_stats(7)
print("\nСтатистика за неделю:")
for day in weekly_stats:
    print(f"  {day}")
```

### 4. Интеграция с дашбордом (Grafana, Metabase, etc.)

Используйте ваш API как источник данных для визуализации:

```python
# Скрипт для экспорта данных в CSV/JSON для дашбордов
import requests
import csv
import json

def export_to_csv(question, filename):
    response = requests.post(
        "http://localhost:3000/api/query",
        json={"question": question}
    )
    data = response.json()['data']
    
    if data:
        with open(filename, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=data[0].keys())
            writer.writeheader()
            writer.writerows(data)
        print(f"✅ Данные экспортированы в {filename}")

# Экспорт различных отчетов
export_to_csv("Топ 10 мерчантов по объему", "top_merchants.csv")
export_to_csv("Транзакции по категориям MCC", "categories.csv")
export_to_csv("Дневной объем за последние 30 дней", "daily_volume.csv")
```

### 5. REST API для мобильных приложений

Ваш backend уже готов для использования в мобильных приложениях:

```dart
// Flutter пример
class PaymentAnalyticsService {
  final String baseUrl = 'http://your-server:3000';
  
  Future<Map<String, dynamic>> query(String question) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/query'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'question': question}),
    );
    
    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    } else {
      throw Exception('Failed to query');
    }
  }
}

// Использование
final service = PaymentAnalyticsService();
final result = await service.query('Сколько транзакций было сегодня?');
print('Результат: ${result['data']}');
```

## 📊 Примеры реальных сценариев

### Сценарий 1: Ежедневный отчет для менеджмента

```bash
#!/bin/bash
# daily_report.sh - Генерирует ежедневный отчет

API_URL="http://localhost:3000/api/query"

echo "📊 Ежедневный отчет - $(date +%Y-%m-%d)"
echo "======================================"

# Общее количество транзакций сегодня
TODAY=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"question": "Сколько транзакций было сегодня?"}' | jq -r '.data[0].total_transactions')

echo "Транзакций сегодня: $TODAY"

# Объем транзакций сегодня
VOLUME=$(curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"question": "Общий объем транзакций сегодня в KZT"}' | jq -r '.data[0].total_volume')

echo "Объем сегодня: $VOLUME KZT"

# Топ категории
echo ""
echo "Топ 5 категорий сегодня:"
curl -s -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{"question": "Топ 5 категорий MCC по объему транзакций сегодня"}' | \
  jq -r '.data[] | "\(.mcc_category): \(.total_volume) KZT"'
```

### Сценарий 2: Мониторинг аномалий

```python
import requests
import time
from datetime import datetime

def check_anomalies():
    """Проверка аномалий в транзакциях"""
    api_url = "http://localhost:3000/api/query"
    
    # Сравниваем сегодня с вчера
    today = requests.post(api_url, json={
        "question": "Сколько транзакций было сегодня?"
    }).json()['data'][0]['total_transactions']
    
    yesterday = requests.post(api_url, json={
        "question": "Сколько транзакций было вчера?"
    }).json()['data'][0]['total_transactions']
    
    change = ((today - yesterday) / yesterday * 100) if yesterday > 0 else 0
    
    if abs(change) > 20:  # Изменение больше 20%
        print(f"⚠️  АНОМАЛИЯ обнаружена!")
        print(f"   Сегодня: {today}")
        print(f"   Вчера: {yesterday}")
        print(f"   Изменение: {change:.1f}%")
        # Отправить уведомление (email, Slack, Telegram и т.д.)

# Запускать каждые 5 минут
while True:
    check_anomalies()
    time.sleep(300)  # 5 минут
```

### Сценарий 3: Интеграция с BI системой

```python
# Периодический экспорт данных для Power BI, Tableau и т.д.
import requests
import pandas as pd
from datetime import datetime, timedelta

def export_for_bi():
    """Экспорт данных для BI систем"""
    api_url = "http://localhost:3000/api/query"
    
    queries = [
        "Транзакции по категориям MCC за последний месяц",
        "Топ 20 мерчантов по объему транзакций",
        "Транзакции по валютам",
        "Дневной объем транзакций за последние 90 дней",
    ]
    
    all_data = []
    for query in queries:
        result = requests.post(api_url, json={"question": query}).json()
        df = pd.DataFrame(result['data'])
        df['report_date'] = datetime.now()
        all_data.append(df)
    
    # Объединяем все данные
    combined_df = pd.concat(all_data, ignore_index=True)
    
    # Сохраняем в Excel для Power BI
    combined_df.to_excel(f"analytics_report_{datetime.now().strftime('%Y%m%d')}.xlsx", index=False)
    print("✅ Отчет экспортирован для BI системы")

export_for_bi()
```

## 🔐 Безопасность и масштабирование

### Добавление аутентификации

Сейчас все запросы анонимные. Для production добавьте:

1. **JWT токены** - для авторизации пользователей
2. **Rate limiting** - ограничение количества запросов
3. **API ключи** - для внешних интеграций
4. **Логирование** - все запросы уже логируются в `query_audit_log`

### Масштабирование

- **Горизонтальное масштабирование**: Запустите несколько инстансов за load balancer
- **Кэширование**: Добавьте Redis для кэширования частых запросов
- **Очереди**: Используйте RabbitMQ/Kafka для асинхронной обработки

## 📈 Метрики и мониторинг

Все запросы логируются в таблицу `query_audit_log`. Можно создать дашборд:

```sql
-- Самые частые вопросы
SELECT question, COUNT(*) as count 
FROM query_audit_log 
GROUP BY question 
ORDER BY count DESC 
LIMIT 10;

-- Среднее время выполнения
SELECT AVG(execution_time_ms) as avg_time 
FROM query_audit_log 
WHERE success = true;

-- Процент успешных запросов
SELECT 
    COUNT(*) FILTER (WHERE success) * 100.0 / COUNT(*) as success_rate
FROM query_audit_log;
```

## 🎯 Итог

Ваша система - это **полноценный production-ready backend**, который можно:

1. ✅ Интегрировать с любым фронтендом (React, Vue, Angular)
2. ✅ Использовать в Telegram ботах
3. ✅ Подключить к BI системам (Power BI, Tableau, Metabase)
4. ✅ Использовать для автоматизации отчетов
5. ✅ Интегрировать в мобильные приложения
6. ✅ Использовать для мониторинга и алертинга

**Следующие шаги:**
- Создайте фронтенд интерфейс
- Настройте Telegram бот
- Добавьте аутентификацию
- Настройте мониторинг и алерты

