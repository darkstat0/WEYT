#!/bin/bash

# NeoVideo Platform Setup Script
# This script sets up the NeoVideo platform without Docker

echo "🚀 Setting up NeoVideo Platform..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python is not installed. Please install Python first."
    exit 1
fi

echo "✅ All required dependencies are installed."

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p uploads
mkdir -p processed
mkdir -p temp
mkdir -p logs
mkdir -p ssl
mkdir -p www/hls
mkdir -p www/dash

# Set up frontend
echo "🎨 Setting up frontend..."
cd frontend
npm install
npm run build
cd ..

# Set up backend
echo "⚙️ Setting up backend..."
cd backend
cargo build --release
cd ..

# Set up AI service
echo "🤖 Setting up AI service..."
cd ai
pip3 install -r requirements.txt
cd ..

echo "✅ Setup complete!"
echo ""
echo "📋 To start the platform:"
echo "   1. Start PostgreSQL database (port 5432)"
echo "   2. Start Redis server (port 6379)"
echo "   3. Start Elasticsearch (port 9200)"
echo "   4. Start ClickHouse (port 8123)"
echo "   5. Run: cd backend && cargo run --release"
echo "   6. Run: cd ai && python3 src/main.py"
echo "   7. Run: cd frontend && npm run dev"
echo ""
echo "🌐 Access the platform at:"
echo "   Frontend: http://localhost:3000"
echo "   Backend API: http://localhost:8080"
echo "   AI Service: http://localhost:5000"
echo ""
echo "📚 For more information, see README.md"