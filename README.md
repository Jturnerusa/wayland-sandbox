# wayland-sandbox

A trivial program to create a wayland security context.

It works by creating a unix socket, setting it to listen, and registering the socket with the
wayland security context API. 

This program does not create a sandbox on its own, and should be used in combination
with bwrap or similar utils.
