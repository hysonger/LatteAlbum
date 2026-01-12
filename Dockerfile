# Multi-stage build for minimal image size
# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app

# Use Alibaba Cloud npm mirror for faster download in mainland China
RUN npm config set registry https://registry.npmmirror.com

# Copy package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Build production frontend
RUN npm run build

# Stage 2: Build backend with Maven
FROM eclipse-temurin:17-jdk-alpine AS backend-builder

WORKDIR /app

# Use Alibaba Cloud mirror for faster download in mainland China
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# Install Maven and configure Alibaba Cloud mirror
RUN apk add --no-cache maven && \
    mkdir -p ~/.m2 && \
    echo '<settings><mirrors><mirror><id>aliyun</id><mirrorOf>central</mirrorOf><name>Aliyun Maven Mirror</name><url>https://maven.aliyun.com/repository/central</url></mirror></mirrors></settings>' > ~/.m2/settings.xml

# Copy frontend build output to Spring Boot static resources
COPY --from=frontend-builder /app/dist ./src/main/resources/static

# Copy pom and source
COPY pom.xml .
RUN mvn dependency:go-offline -B

COPY src ./src

# Build JAR
RUN mvn package -DskipTests -B

# Stage 3: Runtime image
FROM eclipse-temurin:17-jre-alpine

# Use Alibaba Cloud mirror for faster download in mainland China
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# Install runtime dependencies
RUN apk add --no-cache \
    libstdc++ \
    libc6-compat \
    ffmpeg \
    libheif \
    libheif-tools

WORKDIR /app

# Copy jar from builder
COPY --from=backend-builder /app/target/latte-album-*.jar app.jar

# Create directories (Docker volumes will override with proper permissions)
RUN mkdir -p /data/photos /data/db /cache

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/actuator/health || exit 1

# Environment variables (can be overridden)
ENV ALBUM_BASE_PATH=/data/photos
ENV ALBUM_CACHE_DIR=/cache
ENV JAVA_OPTS="-Xmx2g -Xms512m"

# Run application
ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS -jar app.jar"]
