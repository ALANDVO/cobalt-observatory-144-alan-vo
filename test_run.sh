#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /var/lib/alan-portfolio/workspaces/alan/cobalt-observatory-144-alan-vo
cargo test --workspace 2>&1