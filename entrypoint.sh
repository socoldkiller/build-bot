#!/bin/sh
set -e

export PATH="$HOME/bin:$PATH"


PROFILE="$HOME/.profile"
SHRC="$HOME/.shrc"

if [ ! -f "$SHRC" ]; then
  echo "[entrypoint] Creating $SHRC..."
  cat > "$SHRC" <<EOF
alias ll='ls -l'
EOF
fi

if [ ! -f "$PROFILE" ]; then
  echo "[entrypoint] Creating $PROFILE..."
  cat > "$PROFILE" <<EOF
export PATH="\$HOME/bin:\$PATH"
export MY_VAR="hello_from_profile"
[ -f ~/.shrc ] && . ~/.shrc
EOF
fi

. /root/.profile

exec ./build-bot
