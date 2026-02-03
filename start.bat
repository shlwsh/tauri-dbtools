@echo off
echo Starting PostgreSQL Database Tool...

cd /d "%~dp0"

echo Starting backend...
cd backend
if not exist .env (
    echo Creating .env file from .env.example...
    copy .env.example .env
)
start "Backend" cargo run
cd ..

echo Waiting for backend to start...
timeout /t 3 /nobreak >nul

echo Starting frontend...
cd frontend
if not exist node_modules (
    echo Installing dependencies...
    bun install
)
start "Frontend" bun run dev
cd ..

echo.
echo ==========================================
echo PostgreSQL Database Tool is running!
echo ==========================================
echo Backend: http://127.0.0.1:8080
echo Frontend: http://localhost:3000
echo.
echo Press any key to stop all services
echo ==========================================

pause
taskkill /F /IM cargo.exe 2>nul
taskkill /F /IM bun.exe 2>nul
