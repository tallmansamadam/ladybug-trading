# 🐳 DOCKER LAUNCH INSTRUCTIONS

## ⚡ QUICK DOCKER LAUNCH (Recommended)

### Prerequisites
- Docker Desktop installed and running
- Alpaca API credentials in `rust-engine/.env`

### One-Command Launch
```powershell
cd C:\Users\frank\Documents\scripts\trading-scripts\ladybug
docker-compose up -d
```

**That's it!** 🎉

---

## 🔍 VERIFY IT'S RUNNING

### Check Container Status
```powershell
docker-compose ps
```

**Expected Output:**
```
NAME                STATUS              PORTS
ladybug-backend     Up 30 seconds       0.0.0.0:8080->8080/tcp
ladybug-frontend    Up 30 seconds       0.0.0.0:3000->80/tcp
```

### Check Logs
```powershell
# Backend logs
docker-compose logs -f backend

# Frontend logs
docker-compose logs -f frontend

# Both
docker-compose logs -f
```

**Expected Backend Logs:**
```
🐞 LadyBug Trading Engine v0.2.0
Trading ENABLED ✅
Crypto Trading ENABLED ₿✅
API server listening on http://localhost:8080
```

---

## 🌐 ACCESS THE APPLICATION

### Backend API
- **URL:** http://localhost:8080
- **Health Check:** http://localhost:8080/health
- **Status:** http://localhost:8080/status
- **Positions:** http://localhost:8080/positions

### Frontend Dashboard
- **URL:** http://localhost:3000
- Opens in your browser automatically

---

## 🎯 GENERATE TEST DATA

```powershell
# Wait a few seconds for backend to fully start
Start-Sleep -Seconds 10

# Generate test data with live prices
Invoke-RestMethod -Uri "http://localhost:8080/test/generate" -Method Post

# View positions
Invoke-RestMethod -Uri "http://localhost:8080/positions"
```

---

## 🛑 STOP THE CONTAINERS

### Stop (preserves containers)
```powershell
docker-compose stop
```

### Stop and Remove
```powershell
docker-compose down
```

### Stop, Remove, and Clean Volumes
```powershell
docker-compose down -v
```

---

## 🔄 RESTART / REBUILD

### Restart Without Rebuild
```powershell
docker-compose restart
```

### Rebuild and Restart (after code changes)
```powershell
docker-compose up -d --build
```

### Force Complete Rebuild
```powershell
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

---

## 📊 MONITOR LIVE

### Watch Backend Logs
```powershell
docker-compose logs -f backend | Select-String -Pattern "AAPL|BTC|TRADE|SIGNAL"
```

### Watch Container Stats
```powershell
docker stats ladybug-backend ladybug-frontend
```

### Execute Commands in Container
```powershell
# Access backend container shell
docker exec -it ladybug-backend /bin/bash

# Access frontend container shell
docker exec -it ladybug-frontend /bin/sh
```

---

## 🔧 TROUBLESHOOTING

### Port Already in Use
```powershell
# Stop all containers
docker-compose down

# Kill process on port 8080
Stop-Process -Id (Get-NetTCPConnection -LocalPort 8080).OwningProcess -Force

# Restart
docker-compose up -d
```

### Backend Won't Start
```powershell
# Check logs for errors
docker-compose logs backend

# Common issues:
# 1. Missing .env file
# 2. Invalid Alpaca credentials
# 3. Port conflict
```

### Frontend Shows Connection Error
```powershell
# Make sure backend is running
docker-compose ps

# Check backend health
Invoke-RestMethod -Uri "http://localhost:8080/health"

# Restart both
docker-compose restart
```

### Rebuild After Code Changes
```powershell
# Backend code changed
docker-compose up -d --build backend

# Frontend code changed
docker-compose up -d --build frontend

# Both changed
docker-compose up -d --build
```

---

## 🎨 ENVIRONMENT VARIABLES

### Required in `rust-engine/.env`
```env
APCA_API_KEY_ID=your_key_here
APCA_API_SECRET_KEY=your_secret_here
APCA_API_BASE_URL=https://paper-api.alpaca.markets
```

### Optional Docker Environment Variables

Edit `docker-compose.yml`:

```yaml
environment:
  - RUST_LOG=debug  # Change log level (debug, info, warn, error)
  - TRADING_ENABLED=true  # Enable/disable trading
  - CRYPTO_TRADING_ENABLED=true  # Enable/disable crypto
```

---

## 📦 INDIVIDUAL CONTAINER COMMANDS

### Build Backend Only
```powershell
docker build -t ladybug-backend -f Dockerfile .
docker run -d -p 8080:8080 --env-file rust-engine/.env ladybug-backend
```

### Build Frontend Only
```powershell
docker build -t ladybug-frontend -f Dockerfile.frontend .
docker run -d -p 3000:80 ladybug-frontend
```

---

## 🚀 PRODUCTION DEPLOYMENT

### Update docker-compose.yml for Production

```yaml
services:
  backend:
    restart: always  # Change from unless-stopped
    environment:
      - APCA_API_BASE_URL=https://api.alpaca.markets  # Use LIVE API
      - RUST_LOG=warn  # Less verbose logging
    
  frontend:
    restart: always
    environment:
      - REACT_APP_API_URL=https://your-domain.com/api  # Your domain
```

### Behind Reverse Proxy (nginx/Traefik)

```yaml
services:
  backend:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.ladybug-api.rule=Host(`api.yourdomain.com`)"
```

---

## 🔐 SECURITY CONSIDERATIONS

### Environment File Security
```powershell
# Never commit .env files
echo "rust-engine/.env" >> .gitignore

# Use Docker secrets in production
docker secret create alpaca_key rust-engine/.env
```

### Network Isolation
```yaml
# In docker-compose.yml
networks:
  ladybug-network:
    driver: bridge
    internal: true  # No external access
```

---

## 📊 HEALTH CHECKS

### Backend Health
```powershell
Invoke-RestMethod -Uri "http://localhost:8080/health"
```

**Expected:**
```json
{"status": "healthy"}
```

### Container Health Status
```powershell
docker inspect --format='{{.State.Health.Status}}' ladybug-backend
```

**Expected:** `healthy`

---

## 🎯 COMPLETE DOCKER WORKFLOW

### First Time Setup
```powershell
# 1. Clone repo (already done)
cd C:\Users\frank\Documents\scripts\trading-scripts\ladybug

# 2. Ensure .env exists
Test-Path rust-engine/.env

# 3. Build and start
docker-compose up -d

# 4. Wait for startup
Start-Sleep -Seconds 10

# 5. Generate test data
Invoke-RestMethod -Uri "http://localhost:8080/test/generate" -Method Post

# 6. Open browser
Start-Process "http://localhost:3000"
```

### Daily Usage
```powershell
# Start
docker-compose start

# Stop
docker-compose stop
```

### After Code Changes
```powershell
# Pull latest
git pull

# Rebuild and restart
docker-compose up -d --build
```

---

## 🎉 DOCKER ADVANTAGES

✅ **Consistent Environment** - Works the same everywhere  
✅ **Easy Deployment** - One command to start everything  
✅ **Isolated Dependencies** - No conflicts with system packages  
✅ **Easy Updates** - Rebuild and restart  
✅ **Portable** - Run on any machine with Docker  
✅ **Production Ready** - Same containers in dev and prod  

---

## 📝 DOCKER FILES CREATED

- ✅ `Dockerfile` - Backend image
- ✅ `Dockerfile.frontend` - Frontend image
- ✅ `docker-compose.yml` - Orchestration
- ✅ `.dockerignore` - Build optimization
- ✅ `gui/nginx.conf` - Frontend web server

---

## 🚀 QUICK REFERENCE

| Task | Command |
|------|---------|
| Start | `docker-compose up -d` |
| Stop | `docker-compose stop` |
| Logs | `docker-compose logs -f` |
| Rebuild | `docker-compose up -d --build` |
| Status | `docker-compose ps` |
| Remove | `docker-compose down` |
| Shell | `docker exec -it ladybug-backend /bin/bash` |

---

## ✅ VERIFICATION CHECKLIST

- [ ] Docker Desktop running
- [ ] `.env` file exists with Alpaca credentials
- [ ] Run `docker-compose up -d`
- [ ] Check `docker-compose ps` shows both containers running
- [ ] Visit http://localhost:8080/health
- [ ] Visit http://localhost:3000
- [ ] Generate test data
- [ ] View positions
- [ ] Check logs show trading activity

---

**🎉 You're running LadyBug in Docker!**

Everything is containerized, isolated, and production-ready!
