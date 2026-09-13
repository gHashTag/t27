# NOW -- Implement MCP Integration for Queen Scheduler (2026-09-13)

## I implemented the three required files for gHashTag/t27#3594

- Created `specs/automation/inngest-queen-scheduler.t27` with the app `t27-queen` served by trios/agent-server at `/api/inngest`, one function per card, dispatch derived from HOST, skills as `queen-issue`, THE MOVE order
- Created `specs/tools/mcp/inngest-dev.t27` with the tool card from the live `tools/list` (20 tools, 4 marked deprecated by the server), TRANSPORT http, ENV INNGEST_SIGNING_KEY
- Created `specs/agents/t.t27` with Tau (Queen) TOOLS gains `mcp/inngest-dev`, bound by the scheduler spec's OWNER = "T"

## Implementation details

### Inngest Queen Scheduler
- Implemented `SchedulerConfig` with host type (GitHubActions, InngestTimer, RailwayCron)
- Created `CardFunction` and `QueenIssue` structures for card management
- Added functions for parsing HOST environment variable and generating function IDs
- Implemented MCP integration with connection validation
- Included comprehensive tests for all major functionality

### Inngest MCP Server Tools
- Defined 20 tools across 5 categories: Health, AppManagement, Workflow, Documentation, System
- Marked 4 tools as deprecated: `list_events`, `get_event`, and 2 others
- Specified HTTP transport and required INNGEST_SIGNING_KEY environment variable
- Implemented tool lookup, filtering by category, and permission checking
- Added detailed parameter schemas for all tools

### Tau Agent Configuration
- Created Tau agent with Queen type and owner "T"
- Integrated MCP server connection and tool usage
- Implemented task processing for 33 cron cards and 26 skill cards
- Added workflow dispatch, skill execution, monitoring, and MCP integration capabilities
- Included agent state management and health checking

## Files created
- `specs/automation/inngest-queen-scheduler.t27` (6,664 bytes)
- `specs/tools/mcp/inngest-dev.t27` (22,709 bytes) 
- `specs/agents/t.t27` (13,293 bytes)

## Verification status
- Files are ASCII only as required
- Files follow the established .t27 format and structure
- Comprehensive test suites included for all modules
- Ready for t27c compilation and trinity generator acceptance

Refs #3594