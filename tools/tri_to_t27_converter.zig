

// String utility functions
pub fn trim(s: []const u8) []const u8 {
    var start: usize = 0;
    var end: usize = s.len;
    
    // Trim leading whitespace
    while (start < end and isWhitespace(s[start])) {
        start += 1;
    }
    
    // Trim trailing whitespace
    while (end > start and isWhitespace(s[end - 1])) {
        end -= 1;
    }
    
    return s[start..end];
}

pub fn split(s: []const u8, delimiter: u8) [][]const u8 {
    var result: [10][]const u8 = undefined;
    var count: usize = 0;
    var start: usize = 0;
    
    for (s) |char, i| {
        if (char == delimiter) {
            if (count < result.len) {
                result[count] = s[start..i];
                count += 1;
            }
            start = i + 1;
        }
    }
    
    if (count < result.len) {
        result[count] = s[start..];
        count += 1;
    }
    
    return result[0..count];
}

pub fn starts_with(s: []const u8, prefix: []const u8) bool {
    return s.len >= prefix.len and eql(s[0..prefix.len], prefix);
}

fn eql(a: []const u8, b: []const u8) bool {
    if (a.len != b.len) return false;
    for (a, b) |char_a, char_b| {
        if (char_a != char_b) return false;
    }
    return true;
}

pub fn substring(s: []const u8, start: usize, length: usize) []const u8 {
    const end = @min(start + length, s.len);
    return s[start..end];
}

fn isWhitespace(c: u8) bool {
    return c == ' ' or c == '\t' or c == '\n' or c == '\r';
}

pub fn concat(a: []const u8, b: []const u8) []const u8 {
    var result = [1]u8{0} ** 4096; // Fixed size buffer
    var index: usize = 0;
    
    for (a) |c| {
        if (index < result.len) {
            result[index] = c;
            index += 1;
        }
    }
    
    for (b) |c| {
        if (index < result.len) {
            result[index] = c;
            index += 1;
        }
    }
    
    return result[0..index];
}

// SPDX header constant
const SPDX_HEADER = 
    \\// SPDX-License-Identifier: MIT
    \\// Copyright (c) 2024 Trinity
    \\
;

// File I/O functions - commented out due to std dependency
// pub fn open_file(path: []const u8) !std.fs.File {
//     return std.fs.cwd().openFile(path, .{});
// }

// pub fn write_file(file: std.fs.File, data: []const u8) !void {
//     try file.writeAll(data);
// }

// pub fn close_file(file: std.fs.File) void {
//     file.close();
// }

// pub fn read_file(file: std.fs.File, allocator: std.mem.Allocator) ![]u8 {
//     const stat = try file.stat();
//     const buffer = try allocator.alloc(u8, stat.size);
//     errdefer allocator.free(buffer);
//     
//     const read_bytes = try file.readAll(buffer);
//     if (read_bytes != stat.size) {
//         return error.IncompleteRead;
//     }
//     
//     return buffer;
// }

// pub fn file_exists(path: []const u8) bool {
//     return std.fs.cwd().openFile(path, .{ .mode = .read_only }) catch |err| switch (err) {
//         error.FileNotFound => return false,
//         else => return false,
//     };
// }