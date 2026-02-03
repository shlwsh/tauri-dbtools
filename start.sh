#!/bin/bash

echo "Starting PostgreSQL Database Tool..."

cd "$(dirname "$0")"

echo "Starting backend..."
cd backend
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env
fi
cargo run &
BACKEND_PID=$!
cd ..

echo "Waiting for backend to start..."
sleep 3

echo "Starting frontend..."
cd frontend
if [ ! -d node_modules ]; then
    echo "Installing dependencies..."
    bun install
fi
bun run dev &
FRONTEND_PID=$!
cd ..

echo ""
echo "=========================================="
echo "PostgreSQL Database Tool is running!"
echo "=========================================="
echo "Backend: http://127.0.0.1:8080"
echo "Frontend: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop all services"
echo "=========================================="

trap "kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit" SIGINT SIGTERM

wait
