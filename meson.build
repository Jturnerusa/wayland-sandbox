project(
  'wayland-sandbox',
  ['cpp', 'c'],
  version: '0.1.0',
  default_options: ['warning_level=3', 'cpp_std=c++23']
)

wayland = import('unstable-wayland')

wayland_security_context = wayland.scan_xml(
  wayland.find_protocol('security-context', state: 'staging', version: 1)
)

wayland_client = dependency('wayland-client')

sources = files(
  'src/main.cpp'
)

sources += wayland_security_context

executable(
  'wayland-sandbox',
  sources,
  dependencies: wayland_client,
  install: true
)
