#!/bin/ash

echo "Starting blockchain..."

# Controlla se la cartella .dangod esiste
if [ ! -d "$HOME/.dangod" ]; then
    echo "No existing blockchain data found. Initializing..."

    echo "Resetting blockchain..."
    dangod reset

    echo "Generating static node configuration..."
    dangod generate-static

    echo "Building blockchain..."
    dangod build --docker
else
    echo "Existing blockchain data found. Skipping initialization."
fi

# Avvia la blockchain
echo "Starting blockchain..."
dangod start
