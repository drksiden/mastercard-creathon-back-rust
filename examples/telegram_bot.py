#!/usr/bin/env python3
"""
Пример Telegram бота для интеграции с Payment Analytics API
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
            # Показываем первые 3 строки
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

def format_table(data):
    """Форматирует данные в таблицу для Telegram"""
    if not data:
        return "Нет данных"
    
    lines = []
    for i, row in enumerate(data[:10], 1):  # Макс 10 строк
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
        "• Динамика транзакций за неделю"
    )

async def handle_message(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Обработчик текстовых сообщений"""
    question = update.message.text
    
    # Показываем индикатор печати
    await context.bot.send_chat_action(chat_id=update.effective_chat.id, action='typing')
    
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
        await update.message.reply_text("⏱️ Запрос занял слишком много времени. Попробуйте позже.")
    except requests.exceptions.ConnectionError:
        await update.message.reply_text("🔌 Не удалось подключиться к API. Проверьте, запущен ли сервер.")
    except Exception as e:
        await update.message.reply_text(f"❌ Произошла ошибка: {str(e)}")

def main():
    """Запуск бота"""
    application = Application.builder().token(BOT_TOKEN).build()
    
    # Регистрация обработчиков
    application.add_handler(CommandHandler("start", start))
    application.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_message))
    
    print("🤖 Бот запущен...")
    application.run_polling()

if __name__ == "__main__":
    main()

