#! /bin/sh

#enable application logging up to debug level
export RUST_LOG=debug

# make sure default gateway is configured so that we have connectivity to internet
# correctly we should first check:
#   sudo route -n
# but add route action is idempotent so it does not really matter
sudo route add default gw 192.168.1.1

#run the app!
nohup ./garage-controller --config-file ./app_config.toml > /dev/null &
