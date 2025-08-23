#!/bin/bash
set -euo pipefail

echo "Waiting for MongoDB to be ready..."
until mongosh "$MONGO_URL" --eval 'db.runCommand({ ping: 1 })' >/dev/null 2>&1; do
  sleep 1
done

echo "MongoDB is ready!"
echo "Starting Rocket server..."
exec ./apiodactyl
