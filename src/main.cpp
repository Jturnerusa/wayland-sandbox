#include <algorithm>
#include <cerrno>
#include <charconv>
#include <cstdint>
#include <cstring>
#include <expected>
#include <filesystem>
#include <iostream>
#include <optional>
#include <ostream>
#include <security-context-v1-client-protocol.h>
#include <string_view>
#include <sys/socket.h>
#include <sys/un.h>
#include <system_error>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-client.h>

struct App {
  std::optional<wl_display *> display;
  std::optional<wl_registry *> registry;
  std::optional<wp_security_context_v1 *> security_context;
  std::optional<wp_security_context_manager_v1 *> security_context_manager;
};

void register_global(void *data, wl_registry *registry, std::uint32_t name,
                     const char *interface, std::uint32_t version) {
  std::println(std::cerr, "global advertised: {}", interface);

  App *app = static_cast<App *>(data);

  if (std::strcmp(interface, wp_security_context_manager_v1_interface.name) ==
      0) {
    app->security_context_manager =
        static_cast<wp_security_context_manager_v1 *>(wl_registry_bind(
            registry, name, &wp_security_context_manager_v1_interface,
            version));
  }
}

void unregister_global([[gnu::unused]] void *data,
                       [[gnu::unused]] wl_registry *registry,
                       [[gnu::unused]] std::uint32_t name) {}

struct wl_registry_listener listener = {.global = register_global,
                                        .global_remove = unregister_global};

std::optional<int> parse_int(std::string_view string) {
  int number;

  auto [ptr, err] =
      std::from_chars(string.data(), string.data() + string.size(), number);

  if (!(err == std::errc{})) {
    return std::nullopt;
  }

  return number;
}

std::expected<int, int> open_socket(const std::filesystem::path &socket_path) {
  int fd = socket(AF_UNIX, SOCK_STREAM, 0);
  sockaddr_un address = {.sun_family = AF_UNIX, .sun_path = "\0"};
  auto len =
      std::min(sizeof(address.sun_path), std::strlen(socket_path.c_str()));

  std::copy_n(socket_path.c_str(), len, address.sun_path);

  if (bind(fd, reinterpret_cast<sockaddr *>(&address),
           sizeof(address.sun_path)) == -1) {
    return std::unexpected{errno};
  }

  if (listen(fd, 20) == -1) {
    return std::unexpected{errno};
  }

  return fd;
}

int main(int argc, char **argv) {
  if (argc < 2) {
    std::println(
        std::cerr,
        "usage: wayland-sandbox <path to unix socket> <file descriptor>");
    return 1;
  }

  std::filesystem::path socket_path(argv[1]);
  if (std::filesystem::exists(socket_path)) {
    std::println(std::cerr, "socket already exists: {}", argv[1]);
    return 1;
  }

  auto parse_fd_result = parse_int(argv[2]);
  if (!parse_fd_result) {
    std::println(std::cerr, "failed to parse fd: {}", argv[2]);
    return 1;
  }

  int close_fd = parse_fd_result.value();

  App app;

  app.display = wl_display_connect(NULL);

  std::println(std::cerr, "connected to compositor");

  if (!app.display.value()) {
    std::println(std::cerr, "failed to connect to compositor");
    return 1;
  }

  app.registry = wl_display_get_registry(app.display.value());
  wl_registry_add_listener(app.registry.value(), &listener, &app);
  wl_display_roundtrip(app.display.value());

  if (!app.security_context_manager.has_value()) {
    std::println(std::cerr, "failed to bind security context manager");
  }

  auto open_socket_result = open_socket(socket_path);

  if (!open_socket_result) {
    std::println(std::cerr, "failed to open socket: {}",
                 std::strerror(open_socket_result.error()));
    return 1;
  }

  auto sockfd = open_socket_result.value();

  std::println(std::cerr, "opened socket: {}", socket_path.c_str());

  app.security_context = wp_security_context_manager_v1_create_listener(
      app.security_context_manager.value(), sockfd, close_fd);

  wp_security_context_v1_set_sandbox_engine(app.security_context.value(),
                                            "wayland-sandbox");
  wp_security_context_v1_commit(app.security_context.value());

  wl_display_roundtrip(app.display.value());

  return 0;
}
