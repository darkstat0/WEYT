# Multi-stage build for AI Python services
FROM python:3.11-slim AS builder

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements file
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir --user -r requirements.txt

# Copy source code
COPY src ./src
COPY models ./models

# Production stage
FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Create non-root user
RUN groupadd -r aiuser && useradd -r -g aiuser aiuser

# Copy Python from builder stage
COPY --from=builder /root/.local /home/aiuser/.local

# Copy source code and models
COPY --chown=aiuser:aiuser src ./src
COPY --chown=aiuser:aiuser models ./models

# Set environment variables
ENV PATH=/home/aiuser/.local/bin:$PATH
ENV PYTHONPATH=/app/src

# Create directories for models and data
RUN mkdir -p /app/data /app/cache && \
    chown -R aiuser:aiuser /app

# Switch to non-root user
USER aiuser

# Expose port
EXPOSE 5000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

# Run the application
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "5000"]