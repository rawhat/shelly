- generate build "cache" in parallel on first run
  (have to handle some being long af vs others)

- --template ts|js|ex|rb|py|etc

- maybe also write the hashed config for the language to the build dir
  for easier diffing later

- generate `shell.sh` in the project for dropping into shell
  (same as passing `--shell`)
