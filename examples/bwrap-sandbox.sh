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

wayland_sandbox_socket="${XDG_RUNTIME_DIR}/$(mktemp -u wayland-sandbox-XXXXXXX.sock)" || exit $?

close_pipe=$(mktemp -u) || exit $?

mkfifo ${close_pipe} || exit $?

exec {close_fd}<>${close_pipe}

wayland-sandbox  ${wayland_sandbox_socket} ${close_fd} || exit $?

bwrap \
     --unshare-all \
     --die-with-parent \
     --setenv BWRAP 1 \
     --dev /dev \
     --proc /proc \
     --clearenv \
     --setenv XDG_RUNTIME_DIR ${XDG_RUNTIME_DIR} \
     --setenv XDG_SESSION_TYPE wayland \
     --setenv WAYLAND_DISPLAY ${wayland_sandbox_socket} \
     --setenv TERM ${TERM} \
     --setenv COLORTERM ${COLORTERM} \
     --tmpfs /tmp \
     --tmpfs /mnt/sandbox \
     --tmpfs ${HOME} \
     --tmpfs ${XDG_RUNTIME_DIR} \
     --bind ${wayland_sandbox_socket} ${wayland_sandbox_socket} \
     --ro-bind /lib /lib \
     --ro-bind /lib64 /lib64 \
     --ro-bind /bin /bin \
     --ro-bind /sbin /sbin \
     --ro-bind /usr /usr \
     --ro-bind /etc /etc \
     --ro-bind /var /var \
     "$@"

echo 1 > ${close_pipe}
rm ${close_pipe}
rm ${wayland_sandbox_socket}
