# 📊 Таблицы и форматирование данных

## Текущее состояние

### Что возвращает API

```json
{
  "question": "Топ-5 категорий MCC",
  "sql": "SELECT ...",  // Можно скрыть через include_sql=false
  "data": [
    {"mcc_category": "Dining & Restaurants", "count": 523},
    {"mcc_category": "Grocery & Food Markets", "count": 412}
  ],
  "analysis": {
    "headline": "...",
    "data": [...]  // Те же данные для визуализации
  }
}
```

## 🔧 Скрытие SQL в ответе

### По умолчанию SQL показывается (для отладки)

```json
{
  "question": "...",
  "sql": "SELECT ...",  // Показывается
  "data": [...]
}
```

### Скрыть SQL в продакшн

```json
POST /api/query
{
  "question": "Топ-5 категорий",
  "include_sql": false  // Скрыть SQL
}
```

Ответ:
```json
{
  "question": "...",
  // sql отсутствует
  "data": [...]
}
```

## 📋 Форматирование таблиц

### Текущий формат данных

API возвращает данные в JSON формате:

```json
{
  "data": [
    {"mcc_category": "Dining & Restaurants", "count": 523},
    {"mcc_category": "Grocery & Food Markets", "count": 412}
  ]
}
```

### На фронтенде

#### React

```tsx
function DataTable({ data }: { data: any[] }) {
  if (data.length === 0) return <p>Нет данных</p>;
  
  const columns = Object.keys(data[0]);
  
  return (
    <table>
      <thead>
        <tr>
          {columns.map(col => (
            <th key={col}>{col}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {data.map((row, i) => (
          <tr key={i}>
            {columns.map(col => (
              <td key={col}>{row[col]}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

#### Telegram бот

```python
def format_table(data, max_rows=10):
    """Форматирует данные в таблицу для Telegram"""
    if not data:
        return "Нет данных"
    
    lines = []
    for i, row in enumerate(data[:max_rows], 1):
        if isinstance(row, dict):
            row_str = " | ".join([f"{k}: {v}" for k, v in row.items()])
            lines.append(f"{i}. {row_str}")
    
    return "\n".join(lines)
```

### Markdown таблицы (можно добавить endpoint)

Можно добавить endpoint для генерации Markdown таблиц:

```rust
// Будущий endpoint
GET /api/query/{query_id}/table?format=markdown
```

Возвращает:
```markdown
| mcc_category | count |
|--------------|-------|
| Dining & Restaurants | 523 |
| Grocery & Food Markets | 412 |
```

## 🎨 Улучшенное форматирование

### Вариант 1: Endpoint для форматированных таблиц

```rust
// src/api/table.rs
pub async fn format_table(
    data: Vec<serde_json::Value>,
    format: TableFormat,  // Markdown, HTML, CSV
) -> String {
    match format {
        TableFormat::Markdown => format_markdown_table(&data),
        TableFormat::HTML => format_html_table(&data),
        TableFormat::CSV => format_csv_table(&data),
    }
}
```

### Вариант 2: Встроенное форматирование в анализ

Можно добавить в `AnalysisResult`:

```rust
pub struct AnalysisResult {
    // ... существующие поля
    pub formatted_table: Option<String>,  // Markdown таблица
    pub table_html: Option<String>,  // HTML таблица
}
```

## 📊 Примеры использования

### Простая таблица

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true,
    "include_sql": false
  }' | jq '.data'
```

### С форматированием (будущее)

```bash
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Топ-5 категорий MCC",
    "include_analysis": true,
    "include_sql": false,
    "table_format": "markdown"
  }'
```

## 🔄 Текущий workflow

1. **Пользователь задает вопрос** → "Топ-5 категорий MCC"
2. **API генерирует SQL** → `SELECT mcc_category, COUNT(*) ...`
3. **API выполняет запрос** → Получает данные из БД
4. **API возвращает JSON** → `{data: [{mcc_category: "...", count: 123}]}`
5. **Фронтенд/Бот форматирует** → Таблица, диаграмма, текст

## ✅ Рекомендации

### Для продакшн:

1. **Скрыть SQL** - `include_sql: false`
2. **Форматировать на клиенте** - React/Telegram бот
3. **Использовать анализ** - `include_analysis: true` для текста

### Для отладки:

1. **Показать SQL** - `include_sql: true` (по умолчанию)
2. **Проверить данные** - `data` в JSON
3. **Проверить анализ** - `analysis` в JSON

## 📝 Итог

- ✅ **SQL можно скрыть** через `include_sql: false`
- ✅ **Данные в JSON** - легко форматировать на клиенте
- ✅ **Таблицы на клиенте** - React/Telegram бот форматируют
- ⚠️ **Markdown таблицы** - можно добавить endpoint (TODO)

