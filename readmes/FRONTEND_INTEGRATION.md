# 🎨 Интеграция с фронтендом - Быстрый старт

## 📋 Что возвращает API?

### Базовый ответ (без анализа)
```json
{
  "question": "Сколько транзакций?",
  "sql": "SELECT COUNT(*) ...",
  "data": [{"count": 123}],
  "execution_time_ms": 45,
  "row_count": 1,
  "cached": false
}
```

### Расширенный ответ (с анализом)
```json
{
  "question": "Распределение транзакций по валютам",
  "sql": "SELECT ...",
  "data": [
    {"transaction_currency": "KZT", "transaction_count": 11531433},
    {"transaction_currency": "AMD", "transaction_count": 609}
  ],
  "table": "| transaction_currency | transaction_count |\n| --- | --- |\n| KZT | 11531433 |\n| AMD | 609 |",
  "chart_data": {
    "chart_type": "bar",
    "labels": ["KZT", "AMD"],
    "datasets": [{"label": "transaction_count", "data": [11531433.0, 609.0]}]
  },
  "execution_time_ms": 24000,
  "row_count": 11,
  "cached": false,
  "analysis": {
    "headline": "Краткий ответ на вопрос",
    "explanation": "Подробное объяснение результатов...",
    "insights": [
      {
        "title": "Ключевая находка",
        "description": "Детальное описание",
        "significance": "High"
      }
    ],
    "suggested_questions": ["Вопрос 1", "Вопрос 2"],
    "chart_type": "Bar",
    "data": [...]
  }
}
```

## 📊 Готовые таблицы от бэкенда

Бэкенд автоматически генерирует таблицы в **Markdown формате** и возвращает их в поле `table` ответа. Это удобно для:
- Быстрого отображения данных без парсинга JSON
- Экспорта в Markdown-редакторы
- Отображения в Telegram боте
- Использования с библиотеками типа `react-markdown`

### Пример использования готовой таблицы:

```tsx
import ReactMarkdown from 'react-markdown';

// В компоненте
{result.table && (
  <div>
    <h3>Данные</h3>
    <ReactMarkdown>{result.table}</ReactMarkdown>
  </div>
)}
```

Или просто как текст:
```tsx
{result.table && (
  <pre style={{ fontFamily: 'monospace' }}>{result.table}</pre>
)}
```

## 🚀 React интеграция

### 1. Установка зависимостей

```bash
npm install recharts  # Для диаграмм
# или
yarn add recharts
```

### 2. Базовый компонент

```tsx
import React, { useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const API_URL = 'http://localhost:3000/api/query';

interface QueryResponse {
  question: string;
  sql: string;
  data: any[];
  table?: string;  // Готовая Markdown таблица от бэкенда
  chart_data?: {   // Данные для построения диаграммы
    chart_type: string;
    labels: string[];
    datasets: Array<{
      label: string;
      data: number[];
    }>;
  };
  execution_time_ms: number;
  row_count: number;
  analysis?: {
    headline: string;
    explanation: string;
    insights: Array<{
      title: string;
      description: string;
      significance: 'High' | 'Medium' | 'Low';
    }>;
    suggested_questions: string[];
    chart_type?: 'Bar' | 'Line' | 'Pie' | 'Table' | 'Trend';
    data: any[];
  };
  cached: boolean;
}

export function AnalyticsQuery() {
  const [question, setQuestion] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<QueryResponse | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          question,
          include_analysis: true,
          use_cache: true,
        }),
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
    <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Задайте вопрос о данных..."
          style={{ width: '100%', padding: '10px', fontSize: '16px' }}
        />
        <button type="submit" disabled={loading}>
          {loading ? 'Загрузка...' : 'Отправить'}
        </button>
      </form>

      {result && (
        <div style={{ marginTop: '30px' }}>
          {/* Текстовый ответ */}
          {result.analysis && (
            <div style={{ marginBottom: '30px', padding: '20px', background: '#f5f5f5', borderRadius: '8px' }}>
              <h2>{result.analysis.headline}</h2>
              <p>{result.analysis.explanation}</p>
              
              {/* Инсайты */}
              {result.analysis.insights.map((insight, i) => (
                <div key={i} style={{ marginTop: '15px', padding: '10px', background: 'white', borderRadius: '5px' }}>
                  <strong>{insight.title}</strong>
                  <p style={{ margin: '5px 0', color: '#666' }}>{insight.description}</p>
                </div>
              ))}
            </div>
          )}

          {/* Диаграмма */}
          {result.analysis?.chart_type === 'Bar' && result.data.length > 0 && (
            <div style={{ marginBottom: '30px' }}>
              <h3>Диаграмма</h3>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={result.data}>
                  <XAxis dataKey={Object.keys(result.data[0] || {})[0]} />
                  <YAxis />
                  <Tooltip />
                  <Legend />
                  <Bar dataKey={Object.keys(result.data[0] || {})[1] || 'value'} fill="#8884d8" />
                </BarChart>
              </ResponsiveContainer>
            </div>
          )}

          {/* Таблица */}
          {result.data.length > 0 && (
            <div>
              <h3>Данные ({result.row_count} строк)</h3>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ background: '#f0f0f0' }}>
                    {Object.keys(result.data[0]).map(key => (
                      <th key={key} style={{ padding: '10px', border: '1px solid #ddd' }}>
                        {key}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {result.data.map((row, i) => (
                    <tr key={i}>
                      {Object.values(row).map((value, j) => (
                        <td key={j} style={{ padding: '10px', border: '1px solid #ddd' }}>
                          {String(value)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Предложенные вопросы */}
          {result.analysis?.suggested_questions && result.analysis.suggested_questions.length > 0 && (
            <div style={{ marginTop: '30px' }}>
              <h3>Следующие вопросы:</h3>
              {result.analysis.suggested_questions.map((q, i) => (
                <button
                  key={i}
                  onClick={() => setQuestion(q)}
                  style={{ margin: '5px', padding: '8px 15px', cursor: 'pointer' }}
                >
                  {q}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

## 📊 Обработка разных типов диаграмм

```tsx
function renderChart(chartType: string, data: any[]) {
  switch (chartType) {
    case 'Bar':
      return (
        <BarChart data={data}>
          <Bar dataKey="value" />
          <XAxis dataKey="name" />
        </BarChart>
      );
    
    case 'Line':
      return (
        <LineChart data={data}>
          <Line type="monotone" dataKey="value" />
          <XAxis dataKey="date" />
        </LineChart>
      );
    
    case 'Pie':
      return (
        <PieChart>
          <Pie data={data} dataKey="value" nameKey="name" />
        </PieChart>
      );
    
    default:
      return null;
  }
}
```

## 🔄 Контекст и история запросов

**Сейчас:** Каждый запрос независимый, контекст не хранится.

**Для добавления контекста:**

1. **На фронтенде** - храните историю:
```tsx
const [history, setHistory] = useState<QueryResponse[]>([]);

// После каждого запроса
setHistory([...history, result]);
```

2. **На бэкенде** - можно добавить сессии (TODO):
```rust
// Будущая функция
POST /api/query
{
  "question": "...",
  "session_id": "user-123",  // Опционально
  "include_analysis": true
}
```

## 💾 Кэширование

Кэширование работает автоматически:
- Оперативные запросы: 5 минут
- Исторические данные: 30 минут

```tsx
// Включить кэш (по умолчанию false)
{
  "question": "...",
  "use_cache": true
}
```

## 🎯 Примеры использования

### Простой запрос
```tsx
const response = await fetch(API_URL, {
  method: 'POST',
  body: JSON.stringify({
    question: "Сколько транзакций?",
    include_analysis: true
  })
});
```

### С кэшированием
```tsx
const response = await fetch(API_URL, {
  method: 'POST',
  body: JSON.stringify({
    question: "Топ-10 категорий",
    include_analysis: true,
    use_cache: true
  })
});
```

## 📝 Структура данных для визуализации

### Bar Chart
```json
{
  "chart_type": "Bar",
  "data": [
    {"transaction_currency": "KZT", "transaction_count": 11531433},
    {"transaction_currency": "AMD", "transaction_count": 609}
  ]
}
```

**Использование:**
```tsx
<BarChart data={result.data}>
  <XAxis dataKey="transaction_currency" />
  <Bar dataKey="transaction_count" />
</BarChart>
```

### Line Chart
```json
{
  "chart_type": "Line",
  "data": [
    {"date": "2024-01-01", "count": 1000},
    {"date": "2024-01-02", "count": 1200}
  ]
}
```

### Pie Chart
```json
{
  "chart_type": "Pie",
  "data": [
    {"name": "KZT", "value": 11531433},
    {"name": "AMD", "value": 609}
  ]
}
```

## ✅ Ваш запрос - все нормально!

```json
{
  "chart_type": "Bar",  // ✅ Правильно для сравнения категорий
  "data": [
    {"transaction_currency": "KZT", "transaction_count": 11531433},
    // ... 11 валют
  ]
}
```

**Время выполнения 24 секунды** - это нормально для:
- Агрегации по всей таблице (11+ млн записей)
- Группировки по валютам
- Генерации анализа через LLM

**Для ускорения:**
- Используйте кэш: `"use_cache": true`
- Добавьте фильтр: "Распределение по валютам за последний месяц"

## 🚀 Готово к использованию!

Система возвращает:
- ✅ Текстовый ответ (`headline`, `explanation`)
- ✅ Инсайты (`insights`)
- ✅ Данные для таблиц (`data`)
- ✅ Рекомендации диаграмм (`chart_type`)
- ✅ Предложенные вопросы (`suggested_questions`)

Все готово для интеграции в фронтенд!

