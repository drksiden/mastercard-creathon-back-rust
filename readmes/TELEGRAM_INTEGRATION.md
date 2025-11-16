# 🤖 Интеграция с Telegram ботом - Полный гайд

## 📋 Содержание

1. [Быстрый старт](#быстрый-старт)
2. [Установка зависимостей](#установка-зависимостей)
3. [Создание бота](#создание-бота)
4. [Базовый пример](#базовый-пример)
5. [Расширенные возможности](#расширенные-возможности)
6. [Обработка ошибок](#обработка-ошибок)
7. [Форматирование ответов](#форматирование-ответов)
8. [Кэширование](#кэширование)

## 🚀 Быстрый старт

### 1. Создайте бота через @BotFather

1. Откройте Telegram и найдите `@BotFather`
2. Отправьте `/newbot`
3. Следуйте инструкциям для создания бота
4. Сохраните токен (например: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 2. Установите зависимости

```bash
pip install python-telegram-bot requests
# или
pip3 install python-telegram-bot requests
```

### 3. Запустите бота

```bash
python telegram_bot.py
```

## 📦 Установка зависимостей

### Python 3.8+

```bash
# Базовые зависимости
pip install python-telegram-bot requests

# Для форматирования (опционально)
pip install python-dateutil
```

### requirements.txt

```txt
python-telegram-bot==20.7
requests==2.31.0
python-dateutil==2.8.2
```

## 🤖 Создание бота

### Минимальный пример

```python
from telegram import Update
from telegram.ext import Application, CommandHandler, MessageHandler, filters, ContextTypes

BOT_TOKEN = "YOUR_BOT_TOKEN"  # Получите у @BotFather
API_URL = "http://localhost:3000/api/query"

async def start(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await update.message.reply_text(
        "👋 Привет! Я бот для аналитики платежей.\n\n"
        "Задайте вопрос на естественном языке, например:\n"
        "• Сколько транзакций сегодня?\n"
        "• Топ-5 категорий MCC\n"
        "• Динамика транзакций за неделю"
    )

async def handle_message(update: Update, context: ContextTypes.DEFAULT_TYPE):
    question = update.message.text
    
    # Показываем индикатор печати
    await context.bot.send_chat_action(
        chat_id=update.effective_chat.id, 
        action='typing'
    )
    
    try:
        response = requests.post(
            API_URL,
            json={
                "question": question,
                "include_analysis": True,
                "use_cache": True
            },
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            message = format_telegram_response(data)
            await update.message.reply_text(message, parse_mode='Markdown')
        else:
            await update.message.reply_text("❌ Ошибка при обработке запроса")
    except Exception as e:
        await update.message.reply_text(f"❌ Произошла ошибка: {str(e)}")

def main():
    application = Application.builder().token(BOT_TOKEN).build()
    application.add_handler(CommandHandler("start", start))
    application.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_message))
    application.run_polling()

if __name__ == "__main__":
    main()
```

## 📝 Базовый пример

### Полный код бота

```python
#!/usr/bin/env python3
"""
Telegram бот для Payment Analytics API
Требуется: pip install python-telegram-bot requests
"""

import requests
import json
from telegram import Update
from telegram.ext import Application, CommandHandler, MessageHandler, filters, ContextTypes

# Конфигурация
API_URL = "http://localhost:3000/api/query"
BOT_TOKEN = "YOUR_BOT_TOKEN"  # Получите у @BotFather

def format_telegram_response(data):
    """Форматирует ответ API для Telegram"""
    parts = []
    
    if data.get('analysis'):
        # Заголовок
        parts.append(f"📊 *{data['analysis']['headline']}*")
        parts.append("")
        
        # Инсайты (первые 3)
        for insight in data['analysis']['insights'][:3]:
            emoji = "🔴" if insight['significance'] == "High" else \
                   "🟡" if insight['significance'] == "Medium" else "🟢"
            parts.append(f"{emoji} *{insight['title']}*")
            parts.append(f"   {insight['description']}")
            parts.append("")
        
        # Объяснение (первые 500 символов)
        explanation = data['analysis']['explanation']
        if len(explanation) > 500:
            explanation = explanation[:500] + "..."
        parts.append(f"💡 {explanation}")
        
        # Предложенные вопросы
        if data['analysis']['suggested_questions']:
            parts.append("")
            parts.append("❓ *Следующие вопросы:*")
            for q in data['analysis']['suggested_questions'][:2]:
                parts.append(f"   • {q}")
    else:
        # Простой ответ без анализа
        parts.append(f"📊 Результат: {data['row_count']} строк")
        if data.get('data') and len(data['data']) > 0:
            parts.append("")
            parts.append("```")
            for i, row in enumerate(data['data'][:3], 1):
                row_str = " | ".join([f"{k}: {v}" for k, v in row.items()])
                parts.append(f"{i}. {row_str}")
            parts.append("```")
    
    # Метаинформация
    parts.append("")
    parts.append(f"⚡ Время выполнения: {data['execution_time_ms']}ms")
    if data.get('cached'):
        parts.append("💾 Результат из кэша")
    
    return "\n".join(parts)

def format_table(data, max_rows=10):
    """Форматирует данные в таблицу для Telegram"""
    if not data:
        return "Нет данных"
    
    lines = []
    for i, row in enumerate(data[:max_rows], 1):
        if isinstance(row, dict):
            row_str = " | ".join([f"{k}: {v}" for k, v in row.items()])
            lines.append(f"{i}. {row_str}")
    
    return "\n".join(lines) if lines else "Нет данных"

async def start(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Обработчик команды /start"""
    await update.message.reply_text(
        "👋 Привет! Я бот для аналитики платежей.\n\n"
        "Задайте вопрос на естественном языке, например:\n"
        "• Сколько транзакций сегодня?\n"
        "• Топ-5 категорий MCC\n"
        "• Динамика транзакций за неделю\n"
        "• Средний чек по типам транзакций\n\n"
        "Используйте /help для справки"
    )

async def help_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Обработчик команды /help"""
    await update.message.reply_text(
        "📖 *Справка*\n\n"
        "Я могу ответить на вопросы о транзакциях:\n\n"
        "• Количество транзакций\n"
        "• Анализ по категориям\n"
        "• Динамика по времени\n"
        "• Сравнения и статистика\n\n"
        "Просто напишите вопрос на русском языке!",
        parse_mode='Markdown'
    )

async def handle_message(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Обработчик текстовых сообщений"""
    question = update.message.text
    
    # Показываем индикатор печати
    await context.bot.send_chat_action(
        chat_id=update.effective_chat.id, 
        action='typing'
    )
    
    try:
        # Отправляем запрос с анализом
        response = requests.post(
            API_URL,
            json={
                "question": question,
                "include_analysis": True,
                "use_cache": True
            },
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            
            # Формируем и отправляем основной ответ
            message = format_telegram_response(data)
            await update.message.reply_text(message, parse_mode='Markdown')
            
            # Если есть данные для таблицы (больше 1 строки)
            if data.get('data') and len(data['data']) > 1:
                table = format_table(data['data'])
                await update.message.reply_text(
                    f"📋 *Данные:*\n```\n{table}\n```",
                    parse_mode='Markdown'
                )
        else:
            await update.message.reply_text(
                f"❌ Ошибка при обработке запроса (код: {response.status_code})"
            )
            
    except requests.exceptions.Timeout:
        await update.message.reply_text(
            "⏱️ Запрос занял слишком много времени. Попробуйте позже."
        )
    except requests.exceptions.ConnectionError:
        await update.message.reply_text(
            "🔌 Не удалось подключиться к API. Проверьте, запущен ли сервер."
        )
    except Exception as e:
        await update.message.reply_text(f"❌ Произошла ошибка: {str(e)}")

def main():
    """Запуск бота"""
    application = Application.builder().token(BOT_TOKEN).build()
    
    # Регистрация обработчиков
    application.add_handler(CommandHandler("start", start))
    application.add_handler(CommandHandler("help", help_command))
    application.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_message))
    
    print("🤖 Бот запущен...")
    application.run_polling()

if __name__ == "__main__":
    main()
```

## 🎨 Расширенные возможности

### 1. Кнопки для быстрых запросов

```python
from telegram import InlineKeyboardButton, InlineKeyboardMarkup

async def quick_queries(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Показывает кнопки с быстрыми запросами"""
    keyboard = [
        [
            InlineKeyboardButton("📊 Топ-5 категорий", callback_data="query:top5"),
            InlineKeyboardButton("💰 Средний чек", callback_data="query:avg")
        ],
        [
            InlineKeyboardButton("📈 Динамика за неделю", callback_data="query:week"),
            InlineKeyboardButton("🌍 По валютам", callback_data="query:currency")
        ]
    ]
    reply_markup = InlineKeyboardMarkup(keyboard)
    
    await update.message.reply_text(
        "Выберите быстрый запрос:",
        reply_markup=reply_markup
    )

async def button_callback(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Обработчик нажатий на кнопки"""
    query = update.callback_query
    await query.answer()
    
    questions = {
        "top5": "Топ-5 категорий MCC",
        "avg": "Средний чек по типам транзакций",
        "week": "Динамика транзакций за последние 7 дней",
        "currency": "Распределение транзакций по валютам"
    }
    
    question = questions.get(query.data.split(":")[1])
    if question:
        # Отправляем запрос
        await handle_query(update, context, question)
```

### 2. История запросов

```python
# Храним историю в памяти (можно использовать БД)
user_history = {}

async def history_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Показывает историю запросов пользователя"""
    user_id = update.effective_user.id
    
    if user_id not in user_history or not user_history[user_id]:
        await update.message.reply_text("История запросов пуста")
        return
    
    history_text = "📜 *История запросов:*\n\n"
    for i, item in enumerate(user_history[user_id][-5:], 1):  # Последние 5
        history_text += f"{i}. {item['question']}\n"
    
    await update.message.reply_text(history_text, parse_mode='Markdown')

# Сохраняем в handle_message
user_id = update.effective_user.id
if user_id not in user_history:
    user_history[user_id] = []
user_history[user_id].append({
    "question": question,
    "timestamp": datetime.now()
})
```

### 3. Форматирование диаграмм (текстовые)

```python
def format_chart_text(data, chart_type):
    """Форматирует данные для текстовой диаграммы"""
    if chart_type == "Bar":
        # Простая текстовая диаграмма
        max_value = max([row.get('value', 0) or row.get('count', 0) for row in data])
        result = []
        for row in data[:10]:  # Макс 10 элементов
            name = list(row.keys())[0]
            value = row.get('value') or row.get('count', 0)
            bar_length = int((value / max_value) * 20)  # 20 символов максимум
            bar = "█" * bar_length
            result.append(f"{name}: {bar} {value}")
        return "\n".join(result)
    return None
```

### 4. Обработка больших ответов

```python
async def send_large_message(update, text, max_length=4096):
    """Разбивает большое сообщение на части"""
    if len(text) <= max_length:
        await update.message.reply_text(text, parse_mode='Markdown')
    else:
        # Разбиваем на части
        parts = [text[i:i+max_length] for i in range(0, len(text), max_length)]
        for part in parts:
            await update.message.reply_text(part, parse_mode='Markdown')
```

## ⚠️ Обработка ошибок

### Полная обработка ошибок

```python
async def handle_message(update: Update, context: ContextTypes.DEFAULT_TYPE):
    question = update.message.text
    
    await context.bot.send_chat_action(
        chat_id=update.effective_chat.id, 
        action='typing'
    )
    
    try:
        response = requests.post(
            API_URL,
            json={
                "question": question,
                "include_analysis": True,
                "use_cache": True
            },
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            
            # Проверяем на ошибку в SQL
            if data.get('data') and len(data['data']) > 0:
                first_row = data['data'][0]
                if 'error' in first_row:
                    await update.message.reply_text(
                        f"❌ {first_row['error']}\n\n"
                        "Попробуйте переформулировать вопрос, используя:\n"
                        "• Агрегацию (топ-10, количество, сумма)\n"
                        "• Фильтры по времени\n"
                        "• Группировку по категориям"
                    )
                    return
            
            message = format_telegram_response(data)
            await update.message.reply_text(message, parse_mode='Markdown')
            
        elif response.status_code == 500:
            await update.message.reply_text(
                "🔧 Внутренняя ошибка сервера. Попробуйте позже."
            )
        else:
            await update.message.reply_text(
                f"❌ Ошибка {response.status_code}: {response.text}"
            )
            
    except requests.exceptions.Timeout:
        await update.message.reply_text(
            "⏱️ Запрос занял слишком много времени (>30 сек).\n"
            "Попробуйте более простой вопрос или используйте фильтры."
        )
    except requests.exceptions.ConnectionError:
        await update.message.reply_text(
            "🔌 Не удалось подключиться к API.\n"
            "Убедитесь, что сервер запущен на http://localhost:3000"
        )
    except requests.exceptions.JSONDecodeError:
        await update.message.reply_text(
            "❌ Неверный формат ответа от сервера."
        )
    except Exception as e:
        await update.message.reply_text(
            f"❌ Неожиданная ошибка: {str(e)}"
        )
        # Логируем для отладки
        print(f"Error: {e}")
```

## 📊 Форматирование ответов

### Красивое форматирование

```python
def format_telegram_response(data):
    """Улучшенное форматирование для Telegram"""
    parts = []
    
    if data.get('analysis'):
        # Заголовок с эмодзи
        parts.append(f"📊 *{data['analysis']['headline']}*")
        parts.append("")
        
        # Инсайты с цветовыми индикаторами
        for insight in data['analysis']['insights'][:3]:
            emoji = "🔴" if insight['significance'] == "High" else \
                   "🟡" if insight['significance'] == "Medium" else "🟢"
            parts.append(f"{emoji} *{insight['title']}*")
            parts.append(f"   _{insight['description']}_")
            parts.append("")
        
        # Объяснение
        explanation = data['analysis']['explanation']
        if len(explanation) > 500:
            explanation = explanation[:500] + "..."
        parts.append(f"💡 {explanation}")
        
        # Предложенные вопросы как кнопки
        if data['analysis']['suggested_questions']:
            parts.append("")
            parts.append("❓ *Следующие вопросы:*")
            for q in data['analysis']['suggested_questions'][:2]:
                parts.append(f"   • `{q}`")
    else:
        # Простой ответ
        parts.append(f"📊 *Результат:* {data['row_count']} строк")
    
    # Метаинформация
    parts.append("")
    exec_time = data['execution_time_ms']
    if exec_time < 1000:
        time_str = f"{exec_time}ms"
    else:
        time_str = f"{exec_time/1000:.1f}с"
    
    parts.append(f"⚡ Время: {time_str}")
    if data.get('cached'):
        parts.append("💾 Из кэша")
    
    return "\n".join(parts)
```

## 💾 Кэширование

### Использование кэша

```python
# Включить кэш для повторяющихся запросов
response = requests.post(
    API_URL,
    json={
        "question": question,
        "include_analysis": True,
        "use_cache": True  # Включить кэш
    }
)
```

### Очистка кэша (если нужно)

```python
async def clear_cache_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Очищает кэш (требует отдельный endpoint на бэкенде)"""
    # TODO: Добавить endpoint для очистки кэша
    await update.message.reply_text("Кэш очищен (функция в разработке)")
```

## 🚀 Запуск бота

### 1. Сохраните код в `telegram_bot.py`

### 2. Установите токен

```python
BOT_TOKEN = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"  # Ваш токен от @BotFather
```

### 3. Запустите

```bash
python telegram_bot.py
```

### 4. Проверьте

Откройте Telegram, найдите вашего бота и отправьте `/start`

## 📝 Примеры использования

### Пример 1: Простой запрос

```
Пользователь: Сколько транзакций сегодня?
Бот: 📊 Сегодня было обработано 1,523 транзакции...
```

### Пример 2: Аналитический запрос

```
Пользователь: Топ-5 категорий MCC
Бот: 📊 Топ-5 категорий MCC: рестораны лидируют...
     🔴 Доминирование ресторанов
        Категория 'Dining & Restaurants' занимает...
     📋 Данные:
     1. mcc_category: Dining & Restaurants | count: 523
     ...
```

## 🔧 Troubleshooting

### Бот не отвечает

1. Проверьте токен: `BOT_TOKEN` правильный?
2. Проверьте API: сервер запущен на `http://localhost:3000`?
3. Проверьте логи: есть ли ошибки в консоли?

### Ошибки подключения

```python
# Добавьте проверку перед запросом
try:
    test_response = requests.get("http://localhost:3000/api/health", timeout=5)
    if test_response.status_code != 200:
        await update.message.reply_text("⚠️ API недоступен")
        return
except:
    await update.message.reply_text("⚠️ Не удалось подключиться к API")
    return
```

## ✅ Готово!

Теперь у вас есть полнофункциональный Telegram бот для аналитики платежей!

**Следующие шаги:**
- Добавьте команды `/help`, `/history`
- Добавьте кнопки для быстрых запросов
- Настройте обработку ошибок
- Добавьте логирование

