#!/usr/bin/env bash

pkill substrate;
cd ./target/release;
rm -rf db substrate_*;
./substrate purge-chain --dev -y;
python3 start_private_network.py;