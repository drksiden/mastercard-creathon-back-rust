# 🔌 Руководство по интеграции

## 📋 Архитектура

### Использование LLM

**Да, используется одна модель** для обеих задач:
- ✅ **SQL-генерация** - преобразование вопроса в SQL
- ✅ **Анализ результатов** - генерация текста и инсайтов

Модель определяется через `LLM_PROVIDER` в `.env`:
- `gemini` - использует Gemini API
- `ollama` - использует локальный Ollama

**Преимущества:**
- Простая настройка (одна модель)
- Консистентность результатов
- Меньше зависимостей

**Можно разделить позже:**
- Для SQL: быстрая модель (gemini-2.5-flash)
- Для анализа: более мощная модель (gemini-1.5-pro)

## 📤 Формат ответа API

### Базовый ответ (без анализа)

```json
{
  "question": "Сколько транзакций сегодня?",
  "sql": "SELECT COUNT(*) as total FROM transactions WHERE DATE(transaction_timestamp) = CURRENT_DATE;",
  "data": [
    {"total": 1523}
  ],
  "execution_time_ms": 45,
  "row_count": 1,
  "cached": false
}
```

### Расширенный ответ (с анализом)

```json
{
  "question": "Топ-5 категорий MCC",
  "sql": "SELECT mcc_category, COUNT(*) as count FROM transactions GROUP BY mcc_category ORDER BY count DESC LIMIT 5;",
  "data": [
    {"mcc_category": "Dining & Restaurants", "count": 523},
    {"mcc_category": "Grocery & Food Markets", "count": 412},
    {"mcc_category": "Fuel & Service Stations", "count": 298},
    {"mcc_category": "Electronics & Software", "count": 187},
    {"mcc_category": "Travel & Transportation", "count": 156}
  ],
  "execution_time_ms": 234,
  "row_count": 5,
  "cached": false,
  "analysis": {
    "headline": "Топ-5 категорий MCC: рестораны лидируют с 523 транзакциями",
    "insights": [
      {
        "title": "Доминирование ресторанов",
        "description": "Категория 'Dining & Restaurants' занимает первое место с 27% всех транзакций",
        "significance": "High"
      },
      {
        "title": "Высокая активность в продуктовых",
        "description": "Grocery & Food Markets на втором месте с 412 транзакциями",
        "significance": "Medium"
      }
    ],
    "explanation": "Анализ показывает, что наибольшее количество транзакций приходится на категорию 'Dining & Restaurants' (523 транзакции, 27% от общего числа). Это указывает на высокую активность пользователей в ресторанах и кафе. Вторая по популярности категория - 'Grocery & Food Markets' (412 транзакций, 21%), что говорит о регулярных покупках продуктов питания.",
    "suggested_questions": [
      "Показать динамику транзакций по ресторанам за последний месяц",
      "Сравнить средний чек по категориям",
      "Показать топ-10 городов по транзакциям в ресторанах"
    ],
    "chart_type": "Bar",
    "data": [
      {"mcc_category": "Dining & Restaurants", "count": 523},
      {"mcc_category": "Grocery & Food Markets", "count": 412},
      {"mcc_category": "Fuel & Service Stations", "count": 298},
      {"mcc_category": "Electronics & Software", "count": 187},
      {"mcc_category": "Travel & Transportation", "count": 156}
    ]
  }
}
```

## 🤖 Интеграция с Telegram ботом

### Пример на Python

```python
import requests
import json
from telegram import Update
from telegram.ext import Application, CommandHandler, MessageHandler, filters, ContextTypes

API_URL = "http://localhost:3000/api/query"

async def handle_message(update: Update, context: ContextTypes.DEFAULT_TYPE):
    question = update.message.text
    
    # Отправляем запрос с анализом
    response = requests.post(
        API_URL,
        json={
            "question": question,
            "include_analysis": True,
            "use_cache": True
        }
    )
    
    if response.status_code == 200:
        data = response.json()
        
        # Формируем ответ для Telegram
        message = format_telegram_response(data)
        
        # Отправляем текст
        await update.message.reply_text(message, parse_mode='Markdown')
        
        # Если есть данные для таблицы
        if data.get('data') and len(data['data']) > 0:
            table = format_table(data['data'])
            await update.message.reply_text(f"```\n{table}\n```", parse_mode='Markdown')
    else:
        await update.message.reply_text("❌ Ошибка при обработке запроса")

def format_telegram_response(data):
    """Форматирует ответ для Telegram"""
    parts = []
    
    # Заголовок из анализа
    if data.get('analysis'):
        parts.append(f"📊 *{data['analysis']['headline']}*")
        parts.append("")
        
        # Инсайты
        for insight in data['analysis']['insights'][:3]:  # Первые 3
            emoji = "🔴" if insight['significance'] == "High" else "🟡" if insight['significance'] == "Medium" else "🟢"
            parts.append(f"{emoji} *{insight['title']}*")
            parts.append(f"   {insight['description']}")
            parts.append("")
        
        # Объяснение
        parts.append(f"💡 {data['analysis']['explanation'][:500]}...")
        parts.append("")
        
        # Предложенные вопросы
        if data['analysis']['suggested_questions']:
            parts.append("❓ *Следующие вопросы:*")
            for q in data['analysis']['suggested_questions'][:2]:
                parts.append(f"   • {q}")
    else:
        # Простой ответ без анализа
        parts.append(f"📊 Результат: {data['row_count']} строк")
        if data['data']:
            parts.append(f"```\n{json.dumps(data['data'][:3], indent=2, ensure_ascii=False)}\n```")
    
    return "\n".join(parts)

def format_table(data):
    """Форматирует данные в таблицу"""
    if not data:
        return "Нет данных"
    
    # Простая таблица для Telegram
    lines = []
    for i, row in enumerate(data[:10], 1):  # Макс 10 строк
        if isinstance(row, dict):
            row_str = " | ".join([f"{k}: {v}" for k, v in row.items()])
            lines.append(f"{i}. {row_str}")
    
    return "\n".join(lines) if lines else "Нет данных"

# Запуск бота
def main():
    application = Application.builder().token("YOUR_BOT_TOKEN").build()
    application.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_message))
    application.run_polling()

if __name__ == "__main__":
    main()
```

### Пример на Node.js

```javascript
const TelegramBot = require('node-telegram-bot-api');
const axios = require('axios');

const bot = new TelegramBot('YOUR_BOT_TOKEN', {polling: true});
const API_URL = 'http://localhost:3000/api/query';

bot.on('message', async (msg) => {
  const question = msg.text;
  
  try {
    const response = await axios.post(API_URL, {
      question: question,
      include_analysis: true,
      use_cache: true
    });
    
    const data = response.data;
    const message = formatTelegramResponse(data);
    
    await bot.sendMessage(msg.chat.id, message, {parse_mode: 'Markdown'});
    
    // Отправляем таблицу отдельным сообщением
    if (data.data && data.data.length > 0) {
      const table = formatTable(data.data);
      await bot.sendMessage(msg.chat.id, `\`\`\`\n${table}\n\`\`\``, {parse_mode: 'Markdown'});
    }
  } catch (error) {
    await bot.sendMessage(msg.chat.id, '❌ Ошибка при обработке запроса');
  }
});

function formatTelegramResponse(data) {
  const parts = [];
  
  if (data.analysis) {
    parts.push(`📊 *${data.analysis.headline}*`);
    parts.push('');
    
    data.analysis.insights.slice(0, 3).forEach(insight => {
      const emoji = insight.significance === 'High' ? '🔴' : 
                   insight.significance === 'Medium' ? '🟡' : '🟢';
      parts.push(`${emoji} *${insight.title}*`);
      parts.push(`   ${insight.description}`);
      parts.push('');
    });
    
    parts.push(`💡 ${data.analysis.explanation.substring(0, 500)}...`);
  } else {
    parts.push(`📊 Результат: ${data.row_count} строк`);
  }
  
  return parts.join('\n');
}

function formatTable(data) {
  return data.slice(0, 10).map((row, i) => {
    return `${i + 1}. ${Object.entries(row).map(([k, v]) => `${k}: ${v}`).join(' | ')}`;
  }).join('\n');
}
```

## 🌐 Интеграция с фронтендом (React)

### Компонент для запросов

```typescript
// api.ts
export interface QueryRequest {
  question: string;
  include_analysis?: boolean;
  use_cache?: boolean;
}

export interface QueryResponse {
  question: string;
  sql: string;
  data: any[];
  execution_time_ms: number;
  row_count: number;
  analysis?: AnalysisResult;
  cached: boolean;
}

export interface AnalysisResult {
  headline: string;
  insights: Insight[];
  explanation: string;
  suggested_questions: string[];
  chart_type?: 'Bar' | 'Line' | 'Pie' | 'Table' | 'Trend';
  data: any[];
}

export async function queryDatabase(request: QueryRequest): Promise<QueryResponse> {
  const response = await fetch('http://localhost:3000/api/query', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });
  
  if (!response.ok) {
    throw new Error('Query failed');
  }
  
  return response.json();
}
```

### React компонент

```tsx
// QueryComponent.tsx
import React, { useState } from 'react';
import { queryDatabase, QueryResponse } from './api';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts';

export function QueryComponent() {
  const [question, setQuestion] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<QueryResponse | null>(null);
  const [includeAnalysis, setIncludeAnalysis] = useState(true);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    
    try {
      const response = await queryDatabase({
        question,
        include_analysis: includeAnalysis,
        use_cache: true,
      });
      setResult(response);
    } catch (error) {
      console.error('Error:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="query-container">
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Задайте вопрос о данных..."
        />
        <label>
          <input
            type="checkbox"
            checked={includeAnalysis}
            onChange={(e) => setIncludeAnalysis(e.target.checked)}
          />
          Включить анализ
        </label>
        <button type="submit" disabled={loading}>
          {loading ? 'Загрузка...' : 'Отправить'}
        </button>
      </form>

      {result && (
        <div className="result">
          {/* Заголовок */}
          {result.analysis && (
            <div className="analysis">
              <h2>{result.analysis.headline}</h2>
              
              {/* Инсайты */}
              <div className="insights">
                {result.analysis.insights.map((insight, i) => (
                  <div key={i} className={`insight ${insight.significance.toLowerCase()}`}>
                    <h3>{insight.title}</h3>
                    <p>{insight.description}</p>
                  </div>
                ))}
              </div>
              
              {/* Объяснение */}
              <p className="explanation">{result.analysis.explanation}</p>
              
              {/* Диаграмма */}
              {result.analysis.chart_type && result.data.length > 0 && (
                <div className="chart">
                  {result.analysis.chart_type === 'Bar' && (
                    <BarChart width={600} height={300} data={result.data}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis />
                      <Tooltip />
                      <Legend />
                      <Bar dataKey="value" fill="#8884d8" />
                    </BarChart>
                  )}
                </div>
              )}
            </div>
          )}
          
          {/* Таблица данных */}
          <div className="table-container">
            <table>
              <thead>
                <tr>
                  {result.data[0] && Object.keys(result.data[0]).map(key => (
                    <th key={key}>{key}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {result.data.map((row, i) => (
                  <tr key={i}>
                    {Object.values(row).map((value, j) => (
                      <td key={j}>{String(value)}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          
          {/* Предложенные вопросы */}
          {result.analysis?.suggested_questions && (
            <div className="suggested-questions">
              <h3>Следующие вопросы:</h3>
              <ul>
                {result.analysis.suggested_questions.map((q, i) => (
                  <li key={i}>
                    <button onClick={() => setQuestion(q)}>{q}</button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

## 📊 Таблицы и диаграммы

### Текущие возможности

✅ **Что есть:**
- Рекомендация типа диаграммы (`chart_type` в ответе)
- Структурированные данные в JSON формате
- Готовые данные для построения диаграмм

❌ **Что нужно добавить:**
- Генерация изображений диаграмм (можно через библиотеки на фронтенде)
- Markdown таблицы в текстовом формате
- SVG/PNG экспорт

### Генерация таблиц в Markdown

Можно добавить endpoint для генерации Markdown таблиц:

```rust
// Пример функции для генерации Markdown таблицы
fn format_markdown_table(data: &[serde_json::Value]) -> String {
    if data.is_empty() {
        return "Нет данных".to_string();
    }
    
    let mut lines = Vec::new();
    
    // Заголовки
    if let Some(first) = data.first() {
        if let Some(obj) = first.as_object() {
            let headers: Vec<String> = obj.keys().cloned().collect();
            lines.push(format!("| {} |", headers.join(" | ")));
            lines.push(format!("|{}|", headers.iter().map(|_| "---").collect::<Vec<_>>().join("|")));
            
            // Данные
            for row in data.iter().take(20) {
                if let Some(obj) = row.as_object() {
                    let values: Vec<String> = headers.iter()
                        .map(|h| obj.get(h).map(|v| v.to_string()).unwrap_or_default())
                        .collect();
                    lines.push(format!("| {} |", values.join(" | ")));
                }
            }
        }
    }
    
    lines.join("\n")
}
```

### Рекомендации по диаграммам

Система рекомендует тип диаграммы через `chart_type`:

- **Bar** - для сравнения категорий
- **Line** - для временных рядов
- **Pie** - для долей
- **Table** - для детальных данных
- **Trend** - для трендов

**На фронтенде используйте:**
- **Recharts** (React) - для Bar, Line, Pie
- **Chart.js** - универсальная библиотека
- **D3.js** - для кастомных визуализаций

## 🔄 Пример полного цикла

1. **Пользователь задает вопрос** → "Топ-5 категорий MCC"
2. **API генерирует SQL** → `SELECT mcc_category, COUNT(*) ...`
3. **API выполняет запрос** → Получает данные из БД
4. **API анализирует результаты** → Генерирует текст и инсайты
5. **API возвращает JSON** → С данными, анализом и рекомендацией диаграммы
6. **Фронтенд/Бот отображает** → Текст + таблица + диаграмма

## 📝 Резюме

✅ **Одна модель** используется для SQL и анализа  
✅ **JSON формат** ответа - легко интегрировать  
✅ **Структурированные данные** - готовы для таблиц и диаграмм  
✅ **Рекомендации диаграмм** - система подсказывает тип  
⚠️ **Генерация изображений** - нужно делать на фронтенде/боте  

