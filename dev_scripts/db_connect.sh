source .env
psql -h '127.0.0.1' -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}"
