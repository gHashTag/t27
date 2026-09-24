const std = @import("std");

// String utility functions
pub fn trim(s: []const u8) []const u8 {
    var start: usize = 0;
    var end: usize = s.len;
    
    // Trim leading whitespace
    while (start < end and std.mem.trim(u8, s[start..end], " \t\n\r").len == 0) {
        start += 1;
    }
    
    // Trim trailing whitespace
    while (end > start and std.mem.trim(u8, s[start..end], " \t\n\r").len == 0) {
        end -= 1;
    }
    
    return s[start..end];
}

pub fn split(s: []const u8, delimiter: u8) [][]const u8 {
    var result = std.ArrayList([]const u8).init(std.heap.page_allocator);
    defer result.deinit();
    
    var start: usize = 0;
    for (s) |char, i| {
        if (char == delimiter) {
            try result.append(s[start..i]);
            start = i + 1;
        }
    }
    try result.append(s[start..]);
    
    return result.items;
}

pub fn starts_with(s: []const u8, prefix: []const u8) bool {
    return s.len >= prefix.len and std.mem.eql(u8, s[0..prefix.len], prefix);
}

pub fn substring(s: []const u8, start: usize, length: usize) []const u8 {
    const end = @min(start + length, s.len);
    return s[start..end];
}

pub fn concat(a: []const u8, b: []const u8) []const u8 {
    var result = std.ArrayList(u8).init(std.heap.page_allocator);
    defer result.deinit();
    
    try result.appendSlice(a);
    try result.appendSlice(b);
    
    return result.items;
}

// SPDX header constant
const SPDX_HEADER = 
    \\// SPDX-License-Identifier: MIT
    \\// Copyright (c) 2024 Trinity
    \\
;

// File I/O functions
pub fn open_file(path: []const u8) !std.fs.File {
    return std.fs.cwd().openFile(path, .{});
}

pub fn write_file(file: std.fs.File, data: []const u8) !void {
    try file.writeAll(data);
}

pub fn close_file(file: std.fs.File) void {
    file.close();
}

pub fn read_file(file: std.fs.File, allocator: std.mem.Allocator) ![]u8 {
    const stat = try file.stat();
    const buffer = try allocator.alloc(u8, stat.size);
    errdefer allocator.free(buffer);
    
    const read_bytes = try file.readAll(buffer);
    if (read_bytes != stat.size) {
        return error.IncompleteRead;
    }
    
    return buffer;
}

pub fn file_exists(path: []const u8) bool {
    return std.fs.cwd().openFile(path, .{ .mode = .read_only }) catch |err| switch (err) {
        error.FileNotFound => return false,
        else => return false,
    };
}