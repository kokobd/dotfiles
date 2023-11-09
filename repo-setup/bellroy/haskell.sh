#!/usr/bin/env bash

set -x
cd /workspace

echo "use flake" > .envrc
mkdir -p .vscode

if [ ! -f .vscode/settings.json ]; then
cat >.vscode/settings.json <<EOF
{
  "runOnSave.commands": [
    {
      "globMatch": "**/*.hs",
      "command": "ormolu -i \${file}"
    },
    {
      "globMatch": "**/*.cabal",
      "command": "cabal-fmt --inplace \${file}"
    },
    {
      "globMatch": "**/*.dhall",
      "command": "dhall --unicode format \${file}",
      "async": false
    },
    {
      "globMatch": "**/*.dhall",
      "command": "dhall --unicode type --file \${file}",
      "async": false
    },
    {
      "globMatch": "**/*.nix",
      "command": "nixpkgs-fmt \${file}"
    }
  ],
  "files.exclude": {
    "**/.yarn/cache": true,
    "**/dist-newstyle": true
  },
  "files.watcherExclude": {
    "**/.yarn/cache/**": true,
    "**/dist-newstyle": true
  }
}
EOF
fi

if [ ! -f cabal.project.local ]; then
cat >cabal.project.local <<EOF
optimization: False
program-options
  ghc-options: -Wwarn
EOF
fi

# Before settings up my own nix cache, use bellroy's
mkdir -p ~/.local/share/nix
cat >~/.local/share/nix/trusted-settings.json <<EOF
{
  "allow-import-from-derivation": {
    "true": true
  },
  "extra-substituters": {
    "s3://bellroy-nix-cache?profile=bellroy https://cache.iog.io": true
  },
  "extra-trusted-public-keys": {
    "bellroy-nix-cache-1:Cx/qZdMTZiTEUn+B16hIhqvtwYWukKo40EabPBaChJY= hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ=": true
  }
}
EOF
