# 🐳 Docker Setup Guide

## Быстрый старт

### Вариант 1: Локальный Ollama (рекомендуется) ✅

Если у вас уже запущен Ollama на хосте:

```bash
# 1. Убедитесь, что Ollama работает на хосте
ollama list

# 2. В .env укажите (или оставьте по умолчанию):
# OLLAMA_URL=http://localhost:11434  # Для локального запуска
# или
# OLLAMA_URL=http://host.docker.internal:11434  # Для Docker (уже настроено)

# 3. Запустите Docker Compose
docker compose up -d

# 4. Проверьте логи
docker compose logs -f api
```

### Вариант 2: Ollama в Docker

Если хотите запустить Ollama в контейнере:

1. Раскомментируйте сервис `ollama` в `docker-compose.yml`
2. Загрузите модель:
   ```bash
   docker compose exec ollama ollama pull llama3.2
   ```
3. Обновите `OLLAMA_URL` в `.env` на `http://ollama:11434`

## Конфигурация

### Переменные окружения (.env)

```env
# База данных (автоматически настроена в Docker)
DATABASE_URL=postgresql://postgres:password@db:5432/payment_analytics

# LLM Provider
LLM_PROVIDER=ollama

# Ollama (локальный на хосте)
OLLAMA_URL=http://host.docker.internal:11434
OLLAMA_MODEL=mixtral:8x7b-instruct  # или другая модель

# Или Gemini (если используете)
# LLM_PROVIDER=gemini
# GEMINI_API_KEY=your_api_key
# GEMINI_MODEL=gemini-1.5-flash

# Логирование
RUST_LOG=info

# Сервер
HOST=0.0.0.0
PORT=3000
```

## Проверка работы

```bash
# 1. Проверьте статус контейнеров
docker compose ps

# 2. Проверьте логи API
docker compose logs -f api

# Должно быть:
# LLM client initialized: ollama
# LLM ready!
# 🚀 Server running on http://0.0.0.0:3000

# 3. Проверьте health check
curl http://localhost:3000/api/health

# 4. Тестовый запрос
curl -X POST http://localhost:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Сколько транзакций?",
    "include_analysis": true,
    "include_sql": false
  }'
```

## Полезные команды

```bash
# Просмотр логов
docker compose logs -f api
docker compose logs -f db

# Перезапуск API
docker compose restart api

# Остановка всех сервисов
docker compose down

# Остановка с удалением volumes (удалит данные!)
docker compose down -v

# Пересборка образа
docker compose build api

# Удаление orphan контейнеров
docker compose up -d --remove-orphans
```

## Troubleshooting

### API не может подключиться к Ollama на хосте

**Проблема:** `Connection refused` или `Failed to connect to host.docker.internal`

**Решение:**
1. Убедитесь, что Ollama запущен на хосте:
   ```bash
   curl http://localhost:11434/api/tags
   ```

2. Проверьте, что `extra_hosts` настроен в `docker-compose.yml`:
   ```yaml
   extra_hosts:
     - "host.docker.internal:host-gateway"
   ```

3. Для Linux может потребоваться явно указать IP хоста:
   ```bash
   # Узнайте IP хоста
   ip addr show docker0 | grep inet
   
   # Или используйте IP вашей машины
   OLLAMA_URL=http://192.168.1.100:11434
   ```

### API контейнер перезапускается

**Проблема:** Контейнер постоянно перезапускается

**Решение:**
1. Проверьте логи:
   ```bash
   docker compose logs api
   ```

2. Проверьте, что база данных доступна:
   ```bash
   docker compose exec db pg_isready -U postgres
   ```

3. Проверьте переменные окружения:
   ```bash
   docker compose exec api env | grep -E "(DATABASE|OLLAMA|LLM)"
   ```

### Модель не найдена

**Проблема:** `model not found` или `404 Not Found`

**Решение:**
1. Проверьте список моделей:
   ```bash
   # Для локального Ollama
   ollama list
   
   # Для Ollama в Docker
   docker compose exec ollama ollama list
   ```

2. Загрузите модель:
   ```bash
   ollama pull llama3.2
   # или
   docker compose exec ollama ollama pull llama3.2
   ```

3. Обновите `OLLAMA_MODEL` в `.env`

## Структура сервисов

```
┌─────────────────┐
│   API (Rust)    │ ← http://localhost:3000
│   Port: 3000    │
└────────┬────────┘
         │
         ├──→ PostgreSQL (Docker)
         │    Port: 5432
         │
         └──→ Ollama (локальный на хосте)
              Port: 11434
```

## Production Deployment

Для production рекомендуется:

1. Использовать внешнюю базу данных (не в Docker)
2. Настроить reverse proxy (nginx/traefik)
3. Добавить SSL/TLS сертификаты
4. Настроить мониторинг и логирование
5. Использовать secrets management для API ключей

См. `PRODUCTION_DEPLOY.md` для подробностей.
