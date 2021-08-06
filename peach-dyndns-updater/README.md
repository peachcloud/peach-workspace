# peach-dyndns-updater

This is a debian service which uses a systemd timer and nsudpate to keep the IP address of a dynamic dns record up to date. 

It is a simple wrapper for the function  peach_lib::dyndns_client::dyndns_update_ip(),
which reads the PeachCloud configurations from disc, and then if it finds 
that dyndns is enabled, it uses nsupdate to update the IP address of the configured domain records.

The nsupdate requests use the subdomain, dyndns_server_address and a path to a TSIG key (for authentication),
as provided by the PeachCloud configurations. 


## setup

peach-dyndns-udpater is packaged as a debian service, so it can be installed and automatically enabled via:
``` bash
apt-get install peach-dyndns-updater
```

After being installed, it uses a system timer to run the script every five minutes. 

You can see that it is running properly by running:
``` bash
journalctl -u peach-dyndns-udpater
```