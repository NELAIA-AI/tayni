// Minimal HTTP server for benchmark comparison - Zig 0.13
// Compile with: zig build-exe -OReleaseSmall zig_http.zig

const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Zig HTTP Server listening on port 38086...\n", .{});

    // Create socket
    const sock = try std.posix.socket(std.posix.AF.INET, std.posix.SOCK.STREAM, 0);
    defer std.posix.close(sock);

    // Bind
    const addr = std.net.Address.initIp4(.{ 127, 0, 0, 1 }, 38086);
    try std.posix.bind(sock, &addr.any, addr.getOsSockLen());

    // Listen
    try std.posix.listen(sock, 1);

    // Accept
    var client_addr: std.posix.sockaddr = undefined;
    var client_addr_len: std.posix.socklen_t = @sizeOf(std.posix.sockaddr);
    const client = try std.posix.accept(sock, &client_addr, &client_addr_len, 0);
    defer std.posix.close(client);

    // Read request (ignore content)
    var buf: [1024]u8 = undefined;
    _ = try std.posix.read(client, &buf);

    // Send response
    const response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 25\r\n\r\n{\"benchmark\":\"zig\",\"ok\":1}";
    _ = try std.posix.write(client, response);
}
