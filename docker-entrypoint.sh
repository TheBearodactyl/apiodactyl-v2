#!/bin/bash
set -euo pipefail

MONGODB_URL="${MONGODB_URL:-mongodb://db:27017/bearodata}"

echo "Waiting for MongoDB to be ready..."
counter=0
max_attempts=30

until mongosh "$MONGODB_URL" --eval "db.runCommand({ connectionStatus: 1 })" --quiet >/dev/null 2>&1; do
    counter=$((counter+1))
    if [ $counter -ge $max_attempts ]; then
        echo "MongoDB connection failed after $max_attempts attempts"
        exit 1
    fi
    echo "MongoDB is unavailable - sleeping (attempt $counter/$max_attempts)"
    sleep 2
done

echo "MongoDB is ready and connection verified!"
echo "Testing database operations..."
mongosh "$MONGODB_URL" --eval "
    try {
        db.testConnection.insertOne({ timestamp: new Date(), message: 'Connection test' });
        print('Database write test successful');
        db.testConnection.drop();
    } catch (e) {
        print('Database error: ' + e);
        exit(1);
    }
" --quiet

echo "Starting Rocket server..."
exec ./apiodactyl
