#!/usr/bin/sh

set -xe

cp ./mcro.service ~/.config/systemd/user/

systemctl --user restart mcro.service
systemctl --user daemon-reload
journalctl --user-unit mcro.service -f

