@echo off
REM NeoVideo Platform Setup Script for Windows
REM This script sets up the NeoVideo platform without Docker

echo 🚀 Setting up NeoVideo Platform...

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Node.js is not installed. Please install Node.js first.
    pause
    exit /b 1
)

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Rust is not installed. Please install Rust first.
    pause
    exit /b 1
)

REM Check if Python is installed
where python >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Python is not installed. Please install Python first.
    pause
    exit /b 1
)

echo ✅ All required dependencies are installed.

REM Create necessary directories
echo 📁 Creating directories...
if not exist "uploads" mkdir uploads
if not exist "processed" mkdir processed
if not exist "temp" mkdir temp
if not exist "logs" mkdir logs
if not exist "ssl" mkdir ssl
if not exist "www\hls" mkdir www\hls
if not exist "www\dash" mkdir www\dash

REM Set up frontend
echo 🎨 Setting up frontend...
cd frontend
call npm install
call npm run build
cd ..

REM Set up backend
echo ⚙️ Setting up backend...
cd backend
call cargo build --release
cd ..

REM Set up AI service
echo 🤖 Setting up AI service...
cd ai
call pip install -r requirements.txt
cd ..

echo ✅ Setup complete!
echo.
echo 📋 To start the platform:
echo    1. Start PostgreSQL database (port 5432)
echo    2. Start Redis server (port 6379)
echo    3. Start Elasticsearch (port 9200)
echo    4. Start ClickHouse (port 8123)
echo    5. Run: cd backend ^&^& cargo run --release
echo    6. Run: cd ai ^&^& python src/main.py
echo    7. Run: cd frontend ^&^& npm run dev
echo.
echo 🌐 Access the platform at:
echo    Frontend: http://localhost:3000
echo    Backend API: http://localhost:8080
echo    AI Service: http://localhost:5000
echo.
echo 📚 For more information, see README.md
echo.
pause