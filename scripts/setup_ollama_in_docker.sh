#!/bin/bash

# Скрипт для настройки Ollama в Docker

echo "🦙 Настройка Ollama в Docker"
echo "============================"
echo ""

# Проверяем, запущен ли Docker Compose
if ! docker compose ps | grep -q ollama; then
    echo "⚠️  Ollama контейнер не запущен. Запускаю..."
    docker compose up -d ollama
    echo "⏳ Ждем запуска Ollama (10 секунд)..."
    sleep 10
fi

echo ""
echo "📦 Доступные модели Ollama:"
echo "---------------------------"
echo "1. llama3.2 (3GB) - легкая, быстрая"
echo "2. mistral (4GB) - баланс скорости и качества"
echo "3. mixtral:8x7b-instruct (26GB) - мощная, медленная"
echo "4. qwen2.5 (7GB) - хороший баланс"
echo ""

read -p "Какую модель загрузить? (1-4 или введите название): " choice

case $choice in
    1)
        MODEL="llama3.2"
        ;;
    2)
        MODEL="mistral"
        ;;
    3)
        MODEL="mixtral:8x7b-instruct"
        ;;
    4)
        MODEL="qwen2.5"
        ;;
    *)
        MODEL="$choice"
        ;;
esac

echo ""
echo "📥 Загружаю модель: $MODEL"
echo "Это может занять несколько минут..."
echo ""

docker compose exec ollama ollama pull "$MODEL"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Модель $MODEL успешно загружена!"
    echo ""
    echo "📝 Обновите .env файл:"
    echo "   OLLAMA_MODEL=$MODEL"
    echo ""
    echo "🔄 Перезапустите API:"
    echo "   docker compose restart api"
else
    echo ""
    echo "❌ Ошибка при загрузке модели"
    exit 1
fi

