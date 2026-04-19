const std = @import("std");

pub const AgentError = error{
    CellNotFound,
    OrchestratorBusy,
    AgentUnresponsive,
};

pub const AgentId = []const u8;
pub const AgentRole = enum {
    Worker,
    Specialist,
    Scout,
    Coordinator,
};

pub const Cell = struct {
    id: AgentId,
    role: AgentRole,
    state: []const u8,
    active: bool,
};

pub const Orchestrator = struct {
    cells: std.ArrayList(Cell),
    coordinator: ?*Cell,
};

pub fn initOrchestrator(allocator: std.mem.Allocator) !Orchestrator {
    return Orchestrator{
        .cells = std.ArrayList(Cell).init(allocator),
        .coordinator = null,
    };
}

pub fn registerCell(orch: *Orchestrator, cell: Cell) !void {
    try orch.cells.append(cell);
}

pub fn dispatchTask(orch: *Orchestrator, task: []const u8) !AgentId {
    // Round-robin dispatch to available cells
    var best_cell: ?*Cell = null;
    for (orch.cells.items) |cell| {
        if (cell.active) {
            best_cell = cell;
            break;
        }
    }
    return if (best_cell) |c| c.id else error.AgentUnresponsive;
}

test "orchestrator dispatch" {
    const allocator = std.testing.allocator;
    var orch = try initOrchestrator(allocator);
    defer orch.cells.deinit();

    const worker = Cell{
        .id = &[_]u8{1},
        .role = .Worker,
        .state = &[_]u8{},
        .active = true,
    };
    try registerCell(&orch, worker);

    const task = "test_task";
    const dispatched = try dispatchTask(&orch, task);
    try std.testing.expectEqualSlices(u8, dispatched, &[_]u8{1});
}
