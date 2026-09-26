const std = @import("std");
const mem = std.mem;

const AdapterMode = enum {
    Simulation,
    SoftwareConformance,
    FPGAMeasurements,
    FabricatedSiliconMeasurements,
    DryRun,
    RealExecution,
};

const AdapterContract = struct {
    name: []const u8,
    repository: []const u8,
    commit: []const u8,
    profile: []const u8,
    target: []const u8,
    artifact_hash: []const u8,
    prerequisites: [][]const u8,
    is_supported: bool,
    validate: fn(*AdapterContract, []const u8, []const u8, []const u8) anyerror!bool,
    run: fn(*AdapterContract, AdapterMode, [][]const u8) anyerror![]const u8,
};

fn validateTraining(self: *AdapterContract, profile: []const u8, target: []const u8, artifact_hash: []const u8) anyerror!bool {
    if (mem.eql(u8, self.profile, profile) and mem.eql(u8, self.target, target) and mem.eql(u8, self.artifact_hash, artifact_hash)) {
        return true;
    }
    return false;
}

fn runTraining(self: *AdapterContract, mode: AdapterMode, args: [][]const u8) anyerror![]const u8 {
    // Use mode and args to generate evidence
    var buf: [64]u8 = undefined;
    const pos = std.fmt.bufPrint(&buf, "Training evidence: mode={s}, args_len={d}", .{modeToString(mode), args.len}) orelse return "buffer too small";
    return buf[0..pos];
}

fn validateNet(self: *AdapterContract, profile: []const u8, target: []const u8, artifact_hash: []const u8) anyerror!bool {
    if (mem.eql(u8, self.profile, profile) and mem.eql(u8, self.target, target) and mem.eql(u8, self.artifact_hash, artifact_hash)) {
        return true;
    }
    return false;
}

fn runNet(self: *AdapterContract, mode: AdapterMode, args: [][]const u8) anyerror![]const u8 {
    var buf: [64]u8 = undefined;
    const pos = std.fmt.bufPrint(&buf, "Net evidence: mode={s}, args_len={d}", .{modeToString(mode), args.len}) orelse return "buffer too small";
    return buf[0..pos];
}

fn validateFpga(self: *AdapterContract, profile: []const u8, target: []const u8, artifact_hash: []const u8) anyerror!bool {
    if (mem.eql(u8, self.profile, profile) and mem.eql(u8, self.target, target) and mem.eql(u8, self.artifact_hash, artifact_hash)) {
        return true;
    }
    return false;
}

fn runFpga(self: *AdapterContract, mode: AdapterMode, args: [][]const u8) anyerror![]const u8 {
    var buf: [64]u8 = undefined;
    const pos = std.fmt.bufPrint(&buf, "FPGA evidence: mode={s}, args_len={d}", .{modeToString(mode), args.len}) orelse return "buffer too small";
    return buf[0..pos];
}

fn validateCorona(self: *AdapterContract, profile: []const u8, target: []const u8, artifact_hash: []const u8) anyerror!bool {
    if (mem.eql(u8, self.profile, profile) and mem.eql(u8, self.target, target) and mem.eql(u8, self.artifact_hash, artifact_hash)) {
        return true;
    }
    return false;
}

fn runCorona(self: *AdapterContract, mode: AdapterMode, args: [][]const u8) anyerror![]const u8 {
    var buf: [64]u8 = undefined;
    const pos = std.fmt.bufPrint(&buf, "Corona evidence: mode={s}, args_len={d}", .{modeToString(mode), args.len}) orelse return "buffer too small";
    return buf[0..pos];
}

// Define the adapters
const adapters = [_]AdapterContract{
    AdapterContract{
        .name = "trinity-training",
        .repository = "https://github.com/gHashTag/trinity-training",
        .commit = "placeholder_commit_training",
        .profile = "default",
        .target = "cpu",
        .artifact_hash = "placeholder_hash_training",
        .prerequisites = &[_][]const u8{
            "python3>=3.8",
            "torch>=1.0",
        },
        .is_supported = true,
        .validate = validateTraining,
        .run = runTraining,
    },
    AdapterContract{
        .name = "tri-net",
        .repository = "https://github.com/gHashTag/tri-net",
        .commit = "placeholder_commit_net",
        .profile = "default",
        .target = "cpu",
        .artifact_hash = "placeholder_hash_net",
        .prerequisites = &[_][]const u8{
            "libnet>=1.0",
        },
        .is_supported = true,
        .validate = validateNet,
        .run = runNet,
    },
    AdapterContract{
        .name = "trinity-fpga",
        .repository = "https://github.com/gHashTag/trinity-fpga",
        .commit = "placeholder_commit_fpga", // Should be linked to gHashTag/trinity#588
        .profile = "default",
        .target = "fpga",
        .artifact_hash = "placeholder_hash_fpga",
        .prerequisites = &[_][]const u8{
            "fpga_toolchain>=2.0",
            "bitstream_file",
        },
        .is_supported = true,
        .validate = validateFpga,
        .run = runFpga,
    },
    AdapterContract{
        .name = "tt-trinity-corona",
        .repository = "https://github.com/gHashTag/tt-trinity-corona",
        .commit = "placeholder_commit_corona",
        .profile = "default",
        .target = "corona",
        .artifact_hash = "placeholder_hash_corona",
        .prerequisites = &[_][]const u8{
            "corona_sdk>=1.0",
        },
        .is_supported = true,
        .validate = validateCorona,
        .run = runCorona,
    },
};

pub fn main(args: [][]const u8) !void {
    if (args.len < 2) {
        std.debug.print("Usage: {} <command> [args...]\\n", .{args[0]});
        return;
    }

    const command = args[1];

    if (std.mem.eql(u8, command, "list")) {
        listAdapters();
    } else if (std.mem.eql(u8, command, "validate")) {
        if (args.len < 6) {
            std.debug.print("Usage: {} validate <adapter> <profile> <target> <artifact_hash>\\n", .{args[0]});
            return;
        }
        validateAdapter(args[2], args[3], args[4], args[5]);
    } else if (std.mem.eql(u8, command, "run")) {
        if (args.len < 7) {
            std.debug.print("Usage: {} run <adapter> <profile> <target> <artifact_hash> <mode> [args...]\\n", .{args[0]});
            return;
        }
        const mode_str = args[6];
        const mode = parseMode(mode_str) orelse {
            std.debug.print("Invalid mode: {s}\\n", .{mode_str});
            return;
        };
        runAdapter(args[2], args[3], args[4], args[5], mode, args[7..]);
    } else {
        std.debug.print("Unknown command: {s}\\n", .{command});
    }
}

fn listAdapters() !void {
    std.debug.print("Available adapters:\\n", .{});
    for (adapters) |adapter| {
        std.debug.print("  - {s} ({s}@{s})\\n", .{adapter.name, adapter.repository, adapter.commit});
    }
}

fn validateAdapter(adapter_name: []const u8, profile: []const u8, target: []const u8, artifact_hash: []const u8) !void {
    var found: *AdapterContract = undefined;
    for (adapters) |adapter| {
        if (std.mem.eql(u8, adapter.name, adapter_name)) {
            found = &adapter;
            break;
        }
    }

    if (found) |*adapter| {
        if (adapter.is_supported) {
            const valid = try adapter.validate(adapter, profile, target, artifact_hash);
            if (valid) {
                std.debug.print("Adapter {s} validation passed.\\n", .{adapter.name});
            } else {
                std.debug.print("Adapter {s} validation failed: mismatch in profile, target, or artifact hash.\\n", .{adapter.name});
            }
        } else {
            std.debug.print("Adapter {s} is not supported.\\n", .{adapter.name});
        }
    } else {
        std.debug.print("Adapter {s} not found.\\n", .{adapter_name});
    }
}

fn parseMode(mode_str: []const u8) ?AdapterMode {
    const lower = std.mem.toLower(mode_str);
    if (std.mem.eql(u8, lower, "simulation")) {
        return AdapterMode.Simulation;
    } else if (std.mem.eql(u8, lower, "softwareconformance")) {
        return AdapterMode.SoftwareConformance;
    } else if (std.mem.eql(u8, lower, "fpgameasurements")) {
        return AdapterMode.FPGAMeasurements;
    } else if (std.mem.eql(u8, lower, "fabricatedsiliconmeasurements")) {
        return AdapterMode.FabricatedSiliconMeasurements;
    } else if (std.mem.eql(u8, lower, "dryrun")) {
        return AdapterMode.DryRun;
    } else if (std.mem.eql(u8, lower, "reexecution")) {
        return AdapterMode.RealExecution;
    } else {
        return null;
    }
}

fn runAdapter(adapter_name: []const u8, profile: []const u8, target: []const u8, artifact_hash: []const u8, mode: AdapterMode, args: [][]const u8) !void {
    var found: *AdapterContract = undefined;
    for (adapters) |adapter| {
        if (std.mem.eql(u8, adapter.name, adapter_name)) {
            found = &adapter;
            break;
        }
    }

    if (found) |*adapter| {
        if (adapter.is_supported) {
            const valid = try adapter.validate(adapter, profile, target, artifact_hash);
            if (!valid) {
                std.debug.print("Adapter {s} validation failed. Cannot run.\\n", .{adapter.name});
                return;
            }
            // Check prerequisites (simplified: just print them)
            std.debug.print("Checking prerequisites for {s}:\\n", .{adapter.name});
            for (adapter.prerequisites) |prereq| {
                std.debug.print("  - {s}\\n", .{prereq});
            }
            // Run the adapter
            const evidence = try adapter.run(adapter, mode, args);
            std.debug.print("Running {s} in mode {s}:\\n", .{adapter.name, modeToString(mode)});
            std.debug.print("Evidence: {s}\\n", .{evidence});
        } else {
            std.debug.print("Adapter {s} is not supported.\\n", .{adapter.name});
        }
    } else {
        std.debug.print("Adapter {s} not found.\\n", .{adapter_name});
    }
}

fn modeToString(mode: AdapterMode) []const u8 {
    return switch (mode) {
        .AdapterMode.Simulation => "Simulation",
        .AdapterMode.SoftwareConformance => "SoftwareConformance",
        .AdapterMode.FPGAMeasurements => "FPGAMeasurements",
        .AdapterMode.FabricatedSiliconMeasurements => "FabricatedSiliconMeasurements",
        .AdapterMode.DryRun => "DryRun",
        .AdapterMode.RealExecution => "RealExecution",
    };
}