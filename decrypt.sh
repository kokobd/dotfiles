#!/usr/bin/env bash

FILENAME="$1"
rage --decrypt -i ~/.ssh/id_ed25519 -o ${FILENAME%".rage"} $FILENAME
