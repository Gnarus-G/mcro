#!/usr/bin/sh

set -xe

cp ./mcro.service ~/.config/systemd/user/

systemctl --user daemon-reload
systemctl --user enable mcro.service --now
journalctl --user-unit mcro.service -f
