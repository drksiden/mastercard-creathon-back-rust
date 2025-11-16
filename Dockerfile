# Multi-stage build для оптимизации размера образа

# Build stage
FROM rust:latest as builder

WORKDIR /app

# Установка зависимостей для сборки
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копируем файлы зависимостей
COPY Cargo.toml Cargo.lock ./

# Создаем dummy src для кэширования зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Копируем реальный код
COPY src ./src
COPY migrations ./migrations

# Собираем release версию
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник
COPY --from=builder /app/target/release/payment-analytics-backend /app/payment-analytics-backend

# Копируем миграции
COPY migrations ./migrations

# Переменные окружения
ENV RUST_LOG=info
ENV PORT=3000
ENV HOST=0.0.0.0

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["./payment-analytics-backend"]

