#!/bin/bash
set -euo pipefail

echo "Waiting for MongoDB to be ready..."
until mongosh --host db --username bearodactyl --password "ReggieMyLove<3<3" --authenticationDatabase admin --eval "db.runCommand('ping')" --quiet > /dev/null 2>&1; do
  echo "MongoDB is unavailable - sleeping"
  sleep 1
done

echo "MongoDB is ready!"
echo "Starting api server..."
exec ./apiodactyl
