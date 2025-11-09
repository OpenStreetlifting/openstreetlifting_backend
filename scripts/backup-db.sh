#!/bin/bash
set -euo pipefail

# Database Backup Script for OpenStreetLifting
# This script creates PostgreSQL backups and optionally uploads them to multiple locations

# Load environment variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Source .env file if it exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
fi

# Configuration
BACKUP_DIR="${BACKUP_DIR:-$PROJECT_ROOT/backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="openstreetlifting_backup_${TIMESTAMP}.sql"
BACKUP_FILE_COMPRESSED="${BACKUP_FILE}.gz"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_FILE_COMPRESSED"

# Retention settings (days)
LOCAL_RETENTION_DAYS="${LOCAL_RETENTION_DAYS:-7}"
REMOTE_RETENTION_DAYS="${REMOTE_RETENTION_DAYS:-30}"

# Database configuration
DB_USER="${DB_USER:-appuser}"
DB_PASSWORD="${DB_PASSWORD:-apppassword}"
DB_NAME="${DB_NAME:-appdb}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
CONTAINER_NAME="${CONTAINER_NAME:-openstreetlifting_postgres}"

# Remote backup locations (set these in .env or environment)
# S3_BUCKET=""  # e.g., s3://my-bucket/openstreetlifting-backups
# REMOTE_SSH=""  # e.g., user@remote-server.com:/backups/openstreetlifting

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Function to create backup
create_backup() {
    log_info "Starting database backup..."

    # Check if running in Docker
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Using Docker container: $CONTAINER_NAME"
        docker exec -e PGPASSWORD="$DB_PASSWORD" "$CONTAINER_NAME" \
            pg_dump -U "$DB_USER" -d "$DB_NAME" -h localhost -p 5432 \
            --verbose --clean --if-exists --no-owner --no-acl \
            | gzip > "$BACKUP_PATH"
    else
        log_info "Using local PostgreSQL client"
        PGPASSWORD="$DB_PASSWORD" pg_dump \
            -U "$DB_USER" -d "$DB_NAME" -h "$DB_HOST" -p "$DB_PORT" \
            --verbose --clean --if-exists --no-owner --no-acl \
            | gzip > "$BACKUP_PATH"
    fi

    if [ $? -eq 0 ]; then
        BACKUP_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
        log_info "Backup created successfully: $BACKUP_FILE_COMPRESSED ($BACKUP_SIZE)"
    else
        log_error "Backup failed!"
        exit 1
    fi
}

# Function to upload to S3
upload_to_s3() {
    if [ -n "${S3_BUCKET:-}" ]; then
        log_info "Uploading backup to S3: $S3_BUCKET"

        if command -v aws &> /dev/null; then
            aws s3 cp "$BACKUP_PATH" "$S3_BUCKET/$BACKUP_FILE_COMPRESSED"
            if [ $? -eq 0 ]; then
                log_info "Successfully uploaded to S3"
            else
                log_error "Failed to upload to S3"
            fi
        else
            log_warn "AWS CLI not found. Skipping S3 upload."
        fi
    fi
}

# Function to upload via SSH/rsync
upload_to_remote() {
    if [ -n "${REMOTE_SSH:-}" ]; then
        log_info "Uploading backup to remote server: $REMOTE_SSH"

        if command -v rsync &> /dev/null; then
            rsync -avz --progress "$BACKUP_PATH" "$REMOTE_SSH/"
            if [ $? -eq 0 ]; then
                log_info "Successfully uploaded to remote server"
            else
                log_error "Failed to upload to remote server"
            fi
        else
            log_warn "rsync not found. Trying scp..."
            if command -v scp &> /dev/null; then
                scp "$BACKUP_PATH" "$REMOTE_SSH/"
                if [ $? -eq 0 ]; then
                    log_info "Successfully uploaded to remote server"
                else
                    log_error "Failed to upload to remote server"
                fi
            else
                log_warn "scp not found. Skipping remote upload."
            fi
        fi
    fi
}

# Function to cleanup old backups
cleanup_old_backups() {
    log_info "Cleaning up old local backups (older than $LOCAL_RETENTION_DAYS days)..."

    find "$BACKUP_DIR" -name "openstreetlifting_backup_*.sql.gz" -type f -mtime +$LOCAL_RETENTION_DAYS -delete

    log_info "Local cleanup completed"

    # Cleanup old S3 backups if configured
    if [ -n "${S3_BUCKET:-}" ] && command -v aws &> /dev/null; then
        log_info "Cleaning up old S3 backups (older than $REMOTE_RETENTION_DAYS days)..."
        CUTOFF_DATE=$(date -d "$REMOTE_RETENTION_DAYS days ago" +%Y-%m-%d 2>/dev/null || date -v-${REMOTE_RETENTION_DAYS}d +%Y-%m-%d)

        aws s3 ls "$S3_BUCKET/" | while read -r line; do
            BACKUP_DATE=$(echo "$line" | awk '{print $1}')
            BACKUP_NAME=$(echo "$line" | awk '{print $4}')

            if [[ "$BACKUP_DATE" < "$CUTOFF_DATE" ]] && [[ "$BACKUP_NAME" == openstreetlifting_backup_*.sql.gz ]]; then
                log_info "Deleting old S3 backup: $BACKUP_NAME"
                aws s3 rm "$S3_BUCKET/$BACKUP_NAME"
            fi
        done

        log_info "S3 cleanup completed"
    fi
}

# Main execution
main() {
    log_info "=== OpenStreetLifting Database Backup ==="
    log_info "Timestamp: $(date)"
    log_info "Database: $DB_NAME"
    log_info "Backup directory: $BACKUP_DIR"
    echo

    create_backup
    upload_to_s3
    upload_to_remote
    cleanup_old_backups

    echo
    log_info "=== Backup completed successfully ==="
    log_info "Backup file: $BACKUP_PATH"
}

# Run main function
main
