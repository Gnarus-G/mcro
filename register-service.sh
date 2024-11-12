#!/usr/bin/sh

set -xe

cp ./mcro.service ~/.config/systemd/user/

systemctl --user enable mcro.service --now
systemctl --user daemon-reload
journalctl --user-unit mcro.service -f
