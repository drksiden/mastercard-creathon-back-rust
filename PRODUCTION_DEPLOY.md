# 🚀 Деплой в продакшн

## 📋 Подготовка к продакшн

### 1. Сборка release версии

```bash
# Оптимизированная сборка
cargo build --release

# Бинарный файл будет в:
# target/release/payment-analytics-backend
```

### 2. Переменные окружения

Создайте `.env.production`:

```env
# Database
DATABASE_URL=postgresql://user:password@prod-db-host:5432/payment_analytics

# LLM Provider
LLM_PROVIDER=gemini
GEMINI_API_KEY=your-production-key
GEMINI_MODEL=gemini-1.5-flash

# Server
HOST=0.0.0.0
PORT=3000

# Logging
RUST_LOG=info,payment_analytics_backend=warn
```

### 3. Безопасность

- ✅ **Никогда не коммитьте** `.env` файлы
- ✅ Используйте **секреты** (Kubernetes Secrets, AWS Secrets Manager)
- ✅ Настройте **HTTPS** (nginx, Cloudflare)
- ✅ Добавьте **аутентификацию** (JWT токены)
- ✅ Настройте **rate limiting**

## 🐳 Docker деплой

### Dockerfile

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Копируем файлы зависимостей
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Собираем release версию
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник
COPY --from=builder /app/target/release/payment-analytics-backend /app/payment-analytics-backend

# Копируем миграции
COPY migrations ./migrations

# Переменные окружения
ENV RUST_LOG=info
ENV PORT=3000

EXPOSE 3000

CMD ["./payment-analytics-backend"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/payment_analytics
      - LLM_PROVIDER=gemini
      - GEMINI_API_KEY=${GEMINI_API_KEY}
      - GEMINI_MODEL=gemini-1.5-flash
      - RUST_LOG=info
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=payment_analytics
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
```

### Запуск

```bash
# Сборка и запуск
docker-compose up -d

# Просмотр логов
docker-compose logs -f api

# Остановка
docker-compose down
```

## ☁️ Cloud деплой

### AWS (EC2 + RDS)

1. **Создайте EC2 инстанс:**
   ```bash
   # Установите Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Клонируйте репозиторий
   git clone <your-repo>
   cd payment-analytics-backend
   
   # Соберите
   cargo build --release
   
   # Запустите как сервис
   sudo systemctl start payment-analytics
   ```

2. **Systemd service** (`/etc/systemd/system/payment-analytics.service`):
   ```ini
   [Unit]
   Description=Payment Analytics Backend
   After=network.target

   [Service]
   Type=simple
   User=ubuntu
   WorkingDirectory=/home/ubuntu/payment-analytics-backend
   EnvironmentFile=/home/ubuntu/payment-analytics-backend/.env.production
   ExecStart=/home/ubuntu/payment-analytics-backend/target/release/payment-analytics-backend
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

3. **Nginx reverse proxy** (`/etc/nginx/sites-available/payment-analytics`):
   ```nginx
   server {
       listen 80;
       server_name api.yourdomain.com;

       location / {
           proxy_pass http://localhost:3000;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection 'upgrade';
           proxy_set_header Host $host;
           proxy_cache_bypass $http_upgrade;
       }
   }
   ```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-analytics-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: payment-analytics-backend
  template:
    metadata:
      labels:
        app: payment-analytics-backend
    spec:
      containers:
      - name: api
        image: your-registry/payment-analytics-backend:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        - name: GEMINI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secret
              key: api-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: payment-analytics-backend
spec:
  selector:
    app: payment-analytics-backend
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

## 🔒 Безопасность в продакшн

### 1. Аутентификация (TODO)

```rust
// Добавить JWT токены
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation};

pub async fn authenticate(
    headers: HeaderMap,
) -> Result<UserId, AppError> {
    // Проверка JWT токена
}
```

### 2. Rate Limiting

```rust
// Использовать tower-http rate-limit
use tower_http::limit::RateLimitLayer;

let app = Router::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
    // ...
```

### 3. HTTPS

```nginx
server {
    listen 443 ssl;
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:3000;
    }
}
```

## 📊 Мониторинг

### Health check endpoint

Уже есть: `GET /api/health`

### Логирование

```rust
// Используйте structured logging
tracing::info!(
    query = %question,
    sql = %sql,
    execution_time_ms = execution_time,
    "Query executed"
);
```

### Метрики (Prometheus)

```rust
// TODO: Добавить Prometheus метрики
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref QUERY_COUNTER: Counter = Counter::new(
        "queries_total", "Total number of queries"
    ).unwrap();
}
```

## 🚀 Чеклист для продакшн

- [ ] Собрать release версию (`cargo build --release`)
- [ ] Настроить переменные окружения
- [ ] Настроить HTTPS (nginx/Cloudflare)
- [ ] Добавить аутентификацию (JWT)
- [ ] Настроить rate limiting
- [ ] Настроить мониторинг (логи, метрики)
- [ ] Настроить бэкапы БД
- [ ] Настроить автоматический рестарт (systemd/docker)
- [ ] Протестировать под нагрузкой
- [ ] Настроить алерты

## 📝 Примеры запуска

### Локально (для тестирования)

```bash
cargo run --release
```

### Docker

```bash
docker build -t payment-analytics-backend .
docker run -p 3000:3000 --env-file .env.production payment-analytics-backend
```

### Systemd

```bash
sudo systemctl start payment-analytics
sudo systemctl enable payment-analytics
sudo systemctl status payment-analytics
```

