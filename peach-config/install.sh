#!/usr/bin/env bash
echo "deb http://apt.peachcloud.org/ buster main" > /etc/apt/sources.list.d/peach.list
wget -O - http://apt.peachcloud.org/pubkey.gpg | sudo apt-key add -
apt-get update
apt-get install -y peach-config
RUST_LOG=info peach-config setup -i -n -d