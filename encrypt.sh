#!/usr/bin/env bash

rage --encrypt -r "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIM1Btv7KbzDNEMryy2O6lEtkfUzpRpvUhw+pvNlYotBL kokobd@MacBook-Air.local" \
  -o $1.rage $1
rm $1
