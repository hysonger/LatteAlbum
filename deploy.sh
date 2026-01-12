#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Latte Album Docker Deployment Script${NC}"
echo "=========================================="

# Load environment variables from .env if exists
if [ -f .env ]; then
    set -a
    source .env
    set +a
fi

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}[OK] Docker is installed${NC}"

if ! command -v docker-compose &> /dev/null && ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: docker-compose is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}[OK] docker-compose is installed${NC}"

# Create cache directory on host (for easy access to thumbnails)
echo -e "\n${YELLOW}Creating cache directory...${NC}"
mkdir -p cache
echo -e "${GREEN}[OK] Cache directory created${NC}"

# Copy .env if not exists
if [ ! -f .env ]; then
    echo -e "\n${YELLOW}Creating .env file from .env.example...${NC}"
    cp .env.example .env
    echo -e "${GREEN}[OK] .env file created${NC}"
    echo -e "${YELLOW}Please edit .env and set ALBUM_PHOTOS_PATH to your photo directory${NC}"
    exit 0
fi

# Display configuration
SERVER_PORT=${SERVER_PORT:-8080}
echo -e "\n${YELLOW}Configuration:${NC}"
echo "  Server Port: ${SERVER_PORT}"
echo "  Photos Path: ${ALBUM_PHOTOS_PATH:-./photos}"
echo "  Cache: Docker volume (latte_cache)"
echo "  Database: Docker volume (latte_db)"

# Build and deploy
echo -e "\n${YELLOW}Building Docker image...${NC}"
SERVER_PORT=${SERVER_PORT} docker-compose build --no-cache

echo -e "\n${YELLOW}Stopping existing container...${NC}"
SERVER_PORT=${SERVER_PORT} docker-compose down || true

echo -e "\n${YELLOW}Starting container...${NC}"
SERVER_PORT=${SERVER_PORT} docker-compose up -d

echo -e "\n${GREEN}Deployment complete!${NC}"
echo -e "Application: http://localhost:${SERVER_PORT}"
echo -e "Health check: http://localhost:${SERVER_PORT}/actuator/health"
echo -e ""
echo -e "${YELLOW}Volume management:${NC}"
echo -e "  List volumes: ${YELLOW}docker volume ls | grep latte${NC}"
echo -e "  Inspect volume: ${YELLOW}docker volume inspect lattealbum_latte_db${NC}"
echo -e "  Backup database: ${YELLOW}docker run --rm -v lattealbum_latte_db:/data -v \$(pwd)/backup:/backup alpine cp /data/database.db /backup/${NC}"
echo -e ""
echo -e "View logs: ${YELLOW}SERVER_PORT=${SERVER_PORT} docker-compose logs -f${NC}"
echo -e "Stop: ${YELLOW}docker-compose down${NC}"
