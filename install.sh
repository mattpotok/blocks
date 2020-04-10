#!/bin/bash

# Configures `i3blocks` directory and installs all the blocklets.

# Constants
I3BLOCKS_DIR=~/.config/i3blocks
I3BLOCKS_BIN_DIR=$I3BLOCKS_DIR/bin
I3BLOCKS_CFG_DIR=$I3BLOCKS_DIR/cfg

# Configure directory
mkdir -p $I3BLOCKS_DIR
mkdir -p $I3BLOCKS_BIN_DIR
mkdir -p $I3BLOCKS_CFG_DIR
touch $I3BLOCKS_DIR/blocks.log

# Install `battery` blocklet
cd battery
cargo install --path . --root $I3BLOCKS_DIR
cp battery.yaml $I3BLOCKS_CFG_DIR
cd ..

# Install `weather` blocklet
cd weather
cargo install --path . --root $I3BLOCKS_DIR
cp weather.yaml $I3BLOCKS_CFG_DIR
cd ..
