# 🧪 Быстрое тестирование API

## Запуск тестов

### Все тесты сразу
```bash
./test_queries.sh
```

### Только тесты диаграмм
```bash
./test_charts.sh
```

## Ручное тестирование

### 1. Простой запрос с анализом
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько всего транзакций?",
    "include_analysis": true
  }' | jq '.'
```

### 2. Запрос для таблицы
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true
  }' | jq '{headline: .analysis.headline, data: .data, chart_type: .analysis.chart_type}'
```

### 3. Запрос для Bar диаграммы
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций каждого типа?",
    "include_analysis": true
  }' | jq '{chart_type: .analysis.chart_type, data: .data}'
```

### 4. Запрос для Line диаграммы
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Показать динамику транзакций по дням за последние 7 дней",
    "include_analysis": true
  }' | jq '{chart_type: .analysis.chart_type, data: .data}'
```

### 5. Запрос для Pie диаграммы
```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Распределение транзакций по валютам",
    "include_analysis": true
  }' | jq '{chart_type: .analysis.chart_type, data: .data}'
```

## Проверка формата данных

### Проверка структуры ответа
```bash
curl -s -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true
  }' | jq '{
    has_analysis: (.analysis != null),
    has_headline: (.analysis.headline != null),
    has_insights: (.analysis.insights != null),
    has_chart_type: (.analysis.chart_type != null),
    data_count: (.data | length),
    chart_type: .analysis.chart_type
  }'
```

### Вывод только данных для визуализации
```bash
curl -s -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true
  }' | jq '{
    chart_type: .analysis.chart_type,
    data: .data,
    headline: .analysis.headline
  }'
```

## Примеры использования данных

### Для таблицы (React)
```javascript
const response = await fetch('/api/query', {
  method: 'POST',
  body: JSON.stringify({
    question: "Топ-5 категорий MCC",
    include_analysis: true
  })
});
const data = await response.json();

// Рендер таблицы
data.data.forEach(row => {
  console.log(row); // { mcc_category: "...", count: 123 }
});
```

### Для Bar диаграммы (Recharts)
```javascript
// Если chart_type === "Bar"
const chartData = data.data.map(d => ({
  name: d.mcc_category,
  value: d.count
}));

<BarChart data={chartData}>
  <Bar dataKey="value" />
  <XAxis dataKey="name" />
</BarChart>
```

### Для текстового вывода
```javascript
// Headline
console.log(data.analysis.headline);

// Инсайты
data.analysis.insights.forEach(insight => {
  console.log(`${insight.title}: ${insight.description}`);
});

// Объяснение
console.log(data.analysis.explanation);
```

## Ожидаемые результаты

✅ **Успешный ответ должен содержать:**
- `question` - исходный вопрос
- `sql` - сгенерированный SQL
- `data` - массив данных (для таблиц/диаграмм)
- `analysis.headline` - краткий ответ
- `analysis.insights` - массив инсайтов
- `analysis.explanation` - подробное объяснение
- `analysis.chart_type` - тип диаграммы (Bar/Line/Pie/Table/Trend)
- `analysis.suggested_questions` - предложенные вопросы

❌ **Если анализ не работает:**
- Проверьте логи сервера: `cargo run`
- Убедитесь, что `include_analysis: true`
- Проверьте, что LLM провайдер работает (Gemini/Ollama)

