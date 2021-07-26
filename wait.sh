#!/bin/bash
# wait-for-postgres.sh

set -e
  
eval $(cat ./env | sed 's/#.*//g')

host="$1"
shift
  
until PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$host" -U $POSTGRES_USER -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done
  
>&2 echo "Postgres is up - executing command"
exec "$@"
