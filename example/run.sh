#!/bin/bash

# Copyright (C) John Turner 2023
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.

# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

close_pipe=$(mktemp -u)

mkfifo ${close_pipe}

exec {close_fd}<>${close_pipe}

cargo run -- \
      --socket ${XDG_RUNTIME_DIR}/wayland-sandbox \
      --close-fd ${close_fd}

WAYLAND_DISPLAY=${XDG_RUNTIME_DIR}/wayland-sandbox \
               bwrap \
               --unshare-all \
               --die-with-parent \
               --setenv BWRAP 1 \
               --dev /dev \
               --proc /proc \
               --tmpfs /tmp \
               --tmpfs ${HOME} \
               --tmpfs ${XDG_RUNTIME_DIR} \
               --bind ${XDG_RUNTIME_DIR}/wayland-sandbox ${XDG_RUNTIME_DIR}/wayland-sandbox \
               --ro-bind ${HOME}/.config ${HOME}/.config \
               --ro-bind /lib /lib \
               --ro-bind /lib64 /lib64 \
               --ro-bind /bin /bin \
               --ro-bind /sbin /sbin \
               --ro-bind /usr /usr \
               --ro-bind /etc /etc \
               --ro-bind /var /var \
               --tmpfs /mnt/sandbox \
               "$@"

echo 1 > ${close_pipe}
rm ${close_pipe}
rm ${XDG_RUNTIME_DIR}/wayland-sandbox
