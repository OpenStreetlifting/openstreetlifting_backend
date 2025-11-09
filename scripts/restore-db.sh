#!/bin/bash
set -euo pipefail

# Database Restore Script for OpenStreetLifting
# This script restores a PostgreSQL backup

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

# Database configuration
DB_USER="${DB_USER:-appuser}"
DB_PASSWORD="${DB_PASSWORD:-apppassword}"
DB_NAME="${DB_NAME:-appdb}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
CONTAINER_NAME="${CONTAINER_NAME:-openstreetlifting_postgres}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Function to list available backups
list_backups() {
    echo -e "${BLUE}Available backups in $BACKUP_DIR:${NC}"
    echo

    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A "$BACKUP_DIR"/openstreetlifting_backup_*.sql.gz 2>/dev/null)" ]; then
        log_error "No backups found in $BACKUP_DIR"
        exit 1
    fi

    local i=1
    declare -g -a BACKUP_FILES

    for backup in "$BACKUP_DIR"/openstreetlifting_backup_*.sql.gz; do
        if [ -f "$backup" ]; then
            BACKUP_FILES[$i]="$backup"
            local size=$(du -h "$backup" | cut -f1)
            local date=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$backup" 2>/dev/null || stat -c "%y" "$backup" 2>/dev/null | cut -d. -f1)
            echo -e "  ${GREEN}[$i]${NC} $(basename "$backup") - $size - $date"
            ((i++))
        fi
    done
    echo
}

# Function to restore backup
restore_backup() {
    local backup_file="$1"

    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi

    log_warn "WARNING: This will drop and recreate the database '$DB_NAME'"
    log_warn "All current data will be lost!"
    echo
    read -p "Are you sure you want to continue? (yes/no): " confirm

    if [ "$confirm" != "yes" ]; then
        log_info "Restore cancelled"
        exit 0
    fi

    log_info "Starting database restore from: $(basename "$backup_file")"

    # Check if running in Docker
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Using Docker container: $CONTAINER_NAME"

        # Drop existing connections
        log_info "Dropping existing connections..."
        docker exec -e PGPASSWORD="$DB_PASSWORD" "$CONTAINER_NAME" \
            psql -U "$DB_USER" -d postgres -c \
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$DB_NAME' AND pid <> pg_backend_pid();" \
            2>/dev/null || true

        # Restore the backup
        log_info "Restoring backup..."
        gunzip -c "$backup_file" | docker exec -i -e PGPASSWORD="$DB_PASSWORD" "$CONTAINER_NAME" \
            psql -U "$DB_USER" -d "$DB_NAME" -h localhost -p 5432
    else
        log_info "Using local PostgreSQL client"

        # Drop existing connections
        log_info "Dropping existing connections..."
        PGPASSWORD="$DB_PASSWORD" psql -U "$DB_USER" -d postgres -h "$DB_HOST" -p "$DB_PORT" -c \
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$DB_NAME' AND pid <> pg_backend_pid();" \
            2>/dev/null || true

        # Restore the backup
        log_info "Restoring backup..."
        gunzip -c "$backup_file" | PGPASSWORD="$DB_PASSWORD" psql \
            -U "$DB_USER" -d "$DB_NAME" -h "$DB_HOST" -p "$DB_PORT"
    fi

    if [ $? -eq 0 ]; then
        log_info "Database restored successfully!"
    else
        log_error "Database restore failed!"
        exit 1
    fi
}

# Main execution
main() {
    log_info "=== OpenStreetLifting Database Restore ==="
    echo

    # Check if backup file was provided as argument
    if [ $# -eq 1 ]; then
        BACKUP_FILE="$1"

        # If it's not an absolute path, look in BACKUP_DIR
        if [[ "$BACKUP_FILE" != /* ]]; then
            BACKUP_FILE="$BACKUP_DIR/$BACKUP_FILE"
        fi

        restore_backup "$BACKUP_FILE"
    else
        # Interactive mode - list backups and let user choose
        list_backups

        read -p "Select backup to restore (1-${#BACKUP_FILES[@]}): " selection

        if [[ "$selection" =~ ^[0-9]+$ ]] && [ "$selection" -ge 1 ] && [ "$selection" -le "${#BACKUP_FILES[@]}" ]; then
            restore_backup "${BACKUP_FILES[$selection]}"
        else
            log_error "Invalid selection"
            exit 1
        fi
    fi

    echo
    log_info "=== Restore completed ==="
}

# Run main function
main "$@"
