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
cargo install --path battery --root $I3BLOCKS_DIR
cp battery/battery.yaml $I3BLOCKS_CFG_DIR

# Install `simon` blocklet
cargo install --path simon --root $I3BLOCKS_DIR
cp simon/simon.yaml $I3BLOCKS_CFG_DIR


# Install `weather` blocklet
cargo install --path weather --root $I3BLOCKS_DIR
cp weather/weather.yaml $I3BLOCKS_CFG_DIR
