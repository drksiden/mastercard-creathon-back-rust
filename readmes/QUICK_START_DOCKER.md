# 🚀 Быстрый старт после запуска Docker

## ✅ Шаг 1: Загрузите модель Ollama

После запуска Docker Compose нужно загрузить модель:

```bash
# Вариант 1: Автоматический скрипт
./setup_ollama_in_docker.sh

# Вариант 2: Вручную (легкая модель для тестов)
docker compose exec ollama ollama pull llama3.2

# Вариант 3: Мощная модель (если есть место)
docker compose exec ollama ollama pull mixtral:8x7b-instruct
```

**Рекомендация:** Начните с `llama3.2` (3GB) для быстрого тестирования.

## ✅ Шаг 2: Обновите .env

Убедитесь, что в `.env` указано:

```env
LLM_PROVIDER=ollama
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL=llama3.2  # Или другая загруженная модель
```

## ✅ Шаг 3: Перезапустите API

После загрузки модели перезапустите API:

```bash
docker compose restart api
```

## ✅ Шаг 4: Проверьте работу

```bash
# 1. Проверьте health check
curl http://localhost:3000/api/health

# 2. Проверьте логи
docker compose logs -f api

# Должно быть:
# LLM client initialized: ollama
# LLM ready!

# 3. Тестовый запрос
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций?",
    "include_analysis": true,
    "include_sql": false
  }'
```

## 📊 Полезные команды

```bash
# Просмотр логов всех сервисов
docker compose logs -f

# Просмотр логов только API
docker compose logs -f api

# Просмотр логов Ollama
docker compose logs -f ollama

# Список загруженных моделей
docker compose exec ollama ollama list

# Перезапуск API
docker compose restart api

# Остановка всех сервисов
docker compose down

# Остановка с удалением volumes (удалит данные!)
docker compose down -v
```

## 🔍 Troubleshooting

### API не может подключиться к Ollama

```bash
# Проверьте, что Ollama запущен
docker compose ps ollama

# Проверьте логи Ollama
docker compose logs ollama

# Проверьте, что модель загружена
docker compose exec ollama ollama list
```

### Модель не загружается

```bash
# Проверьте доступное место
docker system df

# Попробуйте легкую модель
docker compose exec ollama ollama pull llama3.2
```

### API выдает ошибки

```bash
# Проверьте логи
docker compose logs api

# Проверьте .env файл
cat .env | grep OLLAMA

# Перезапустите API
docker compose restart api
```

## ✅ Готово!

После выполнения всех шагов система должна работать:
- ✅ API доступен на http://localhost:3000
- ✅ Ollama работает в контейнере
- ✅ PostgreSQL работает в контейнере
- ✅ Модель загружена и готова к использованию

