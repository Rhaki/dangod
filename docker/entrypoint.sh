#!/bin/ash

echo "Starting blockchain with mode: $1"

if [ "$1" = "reset" ]; then
    echo "Resetting blockchain..."
    dangod reset

    echo "Generating static node configuration..."
    dangod generate-static

    echo "Building blockchain..."
    dangod build
fi

echo "Starting blockchain..."
dangod start
