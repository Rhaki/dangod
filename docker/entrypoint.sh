#!/bin/ash

echo "Starting blockchain..."

HYPERLANE_DOMAIN=""

while [[ $# -gt 0 ]]; do
    case "$1" in
    --hyperlane_domain)
        HYPERLANE_DOMAIN="--hyperlane_domain $2"
        shift 2
        ;;
    *)
        shift
        ;;
    esac
done

# If .dangod directory does not exist, initialize the blockchain
if [ ! -d "$HOME/.dangod" ]; then
    echo "No existing blockchain data found. Initializing..."

    echo "Resetting blockchain..."
    dangod reset

    echo "Generating static node configuration..."
    dangod generate-static

    echo "Building blockchain..."
    dangod build --docker $HYPERLANE_DOMAIN
else
    echo "Existing blockchain data found. Skipping initialization."
fi

# Start the blockchain
echo "Starting blockchain..."
dangod start
