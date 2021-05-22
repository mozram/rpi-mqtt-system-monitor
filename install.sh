#!/bin/bash

echo "Entering directory..."
cd monitoring-service

# Build
echo "Building..."
cargo build --release

# Stop service before copy
echo "Stoping service..."
systemctl stop monitoring-service

echo "Installing..."
cp target/release/monitoring-service /usr/bin

echo "Restarting service..."
systemctl daemon-reload
systemctl start monitoring-service
