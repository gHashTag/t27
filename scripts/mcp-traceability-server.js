#!/usr/bin/env node
/**
 * MCP Server for t27 Traceability Enforcement
 * Integrates with Claude Code hooks for L1 TRACEABILITY enforcement
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

// Configuration
const PROJECT_ROOT = process.env.PROJECT_ROOT || process.cwd();
const ENFORCE_L1 = process.env.ENFORCE_L1 === "true";

// Create MCP server
const server = new Server(
  {
    name: "t27-traceability",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
      resources: {},
    },
  }
);

// Tool: Check L1 TRACEABILITY compliance
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "check_traceability",
        description:
          "Check if a commit message or PR description complies with L1 TRACEABILITY (Closes #N required)",
        inputSchema: {
          type: "object",
          properties: {
            message: {
              type: "string",
              description: "The commit message or PR description to check",
            },
          },
          required: ["message"],
        },
      },
      {
        name: "generate_commit_message",
        description:
          "Generate a compliant commit message for t27 including required L1 TRACEABILITY reference",
        inputSchema: {
          type: "object",
          properties: {
            type: {
              type: "string",
              description: "Commit type: feat, fix, docs, refactor, test, chore",
              enum: ["feat", "fix", "docs", "refactor", "test", "chore"],
            },
            scope: {
              type: "string",
              description: "Commit scope (e.g., compiler, ternary, docs)",
            },
            description: {
              type: "string",
              description: "Brief description of changes",
            },
            issueNumber: {
              type: "string",
              description: "Issue number to reference (e.g., 42)",
            },
            invariantLaws: {
              type: "array",
              description: "Relevant Invariant Laws (L1-L7)",
              items: {
                type: "string",
              },
            },
            body: {
              type: "string",
              description: "Detailed body of the commit message",
            },
          },
          required: ["type", "scope", "description", "issueNumber"],
        },
      },
      {
        name: "validate_branch_name",
        description:
          "Validate branch name follows t27 naming conventions",
        inputSchema: {
          type: "object",
          properties: {
            branchName: {
              type: "string",
              description: "Branch name to validate",
            },
          },
          required: ["branchName"],
        },
      },
      {
        name: "get_phi_loop_template",
        description:
          "Get PHI LOOP stacked branch template for ring development",
        inputSchema: {
          type: "object",
          properties: {
            ringNumber: {
              type: "string",
              description: "Ring number (e.g., 32)",
            },
          },
          required: ["ringNumber"],
        },
      },
    ],
  };
});

// Tool: Check L1 TRACEABILITY compliance
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "check_traceability": {
        const message = args.message as string;
        const hasIssueRef = /(Closes #|Fixes #|Resolves #)/i.test(message);
        const issueMatch = message.match(/(Closes #|Fixes #|Resolves #)(\d+)/i);

        if (!hasIssueRef) {
          return {
            content: [
              {
                type: "text",
                text: JSON.stringify(
                  {
                    compliant: false,
                    error: "L1 TRACEABILITY violation: No issue reference found",
                    required: "Must include 'Closes #N', 'Fixes #N', or 'Resolves #N'",
                    enforce: ENFORCE_L1,
                  },
                  null,
                  2
                ),
              },
            ],
          };
        }

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  compliant: true,
                  issueRef: issueMatch ? issueMatch[0] : null,
                  issueNumber: issueMatch ? issueMatch[2] : null,
                  enforce: ENFORCE_L1,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "generate_commit_message": {
        const type = args.type as string;
        const scope = args.scope as string;
        const description = args.description as string;
        const issueNumber = args.issueNumber as string;
        const invariantLaws = (args.invariantLaws as string[]) || [];
        const body = (args.body as string) || "";

        const commitMessage = `${type}(${scope}): ${description}\n`;

        let fullMessage = commitMessage;

        if (body) {
          fullMessage += `\n${body}\n`;
        }

        if (invariantLaws.length > 0) {
          fullMessage += `\n${invariantLaws
            .map((law) => `${law}: See docs/T27-CONSTITUTION.md`)
            .join("\n")}\n`;
        }

        fullMessage += `\nCloses #${issueNumber}`;

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  commitMessage: fullMessage,
                  compliant: true,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "validate_branch_name": {
        const branchName = args.branchName as string;

        const patterns = {
          feature: /^feat\/.+$/,
          fix: /^fix\/.+$/,
          docs: /^docs\/.+$/,
          test: /^test\/.+$/,
          refactor: /^refactor\/.+$/,
          chore: /^chore\/.+$/,
          ring: /^ring-\d{3}-.+$/,
        };

        let valid = false;
        let matchedPattern = null;

        for (const [patternName, regex] of Object.entries(patterns)) {
          if (regex.test(branchName)) {
            valid = true;
            matchedPattern = patternName;
            break;
          }
        }

        // Additional checks
        const warnings = [];
        if (branchName.includes("ru") && branchName.startsWith("docs/")) {
          warnings.push(
            "LANG-EN violation: Branch name suggests Russian documentation"
          );
        }
        if (/^dv-/.test(branchName)) {
          warnings.push(
            "Non-standard branch name pattern (dv- prefix not recognized)"
          );
        }

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  valid,
                  matchedPattern,
                  warnings,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      case "get_phi_loop_template": {
        const ringNumber = args.ringNumber as string;
        const paddedRing = ringNumber.padStart(3, "0");

        const phases = [
          { name: "issue", branch: `ring-${paddedRing}-issue`, description: "Create and define issue" },
          { name: "spec", branch: `ring-${paddedRing}-spec`, description: "Write .t27 specifications", depends_on: "issue" },
          { name: "tdd", branch: `ring-${paddedRing}-tdd`, description: "Write TDD tests", depends_on: "spec" },
          { name: "code", branch: `ring-${paddedRing}-code`, description: "Implement code", depends_on: "tdd" },
          { name: "gen", branch: `ring-${paddedRing}-gen`, description: "Generate code from specs", depends_on: "code" },
          { name: "seal", branch: `ring-${paddedRing}-seal`, description: "Create verification seals", depends_on: "gen" },
          { name: "verify", branch: `ring-${paddedRing}-verify`, description: "Verify conformance", depends_on: "seal" },
          { name: "land", branch: `ring-${paddedRing}-land`, description: "Land to main branch", depends_on: "verify" },
          { name: "learn", branch: `ring-${paddedRing}-learn`, description: "Document learnings", depends_on: "land" },
        ];

        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(
                {
                  ring: ringNumber,
                  branches: phases,
                  gitCommands: phases
                    .map(
                      (phase) =>
                        `# Phase ${phase.name}: ${phase.description}\n` +
                        `but branch create ${phase.branch} --from ${phase.depends_on ? `ring-${paddedRing}-${phase.depends_on}` : "dev"}\n` +
                        `but apply ${phase.branch}\n\n`
                    )
                    .join(""),
                  summary: `PHI LOOP for Ring ${ringNumber} with 9 stacked branches`,
                },
                null,
                2
              ),
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(
            {
              error: `Tool execution failed: ${error}`,
            },
            null,
            2
          ),
        },
      ],
      isError: true,
    };
  }
});

// Resources
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: [
      {
        uri: `file://${PROJECT_ROOT}/docs/T27-CONSTITUTION.md`,
        name: "T27 Constitution",
        description: "Constitutional laws including L1 TRACEABILITY",
        mimeType: "text/markdown",
      },
      {
        uri: `file://${PROJECT_ROOT}/docs/l1-traceability-audit.md`,
        name: "L1 TRACEABILITY Audit",
        description: "Audit report for L1 TRACEABILITY compliance",
        mimeType: "text/markdown",
      },
    ],
  };
});

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const url = request.params.uri.toString();
  const fs = await import("fs/promises");

  try {
    const filePath = url.replace(`file://${PROJECT_ROOT}/`, "");
    const content = await fs.readFile(filePath, "utf-8");

    return {
      contents: [
        {
          uri: url,
          mimeType: "text/markdown",
          text: content,
        },
      ],
    };
  } catch (error) {
    throw new Error(`Failed to read resource: ${error}`);
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);

  console.error(
    JSON.stringify({
      level: "info",
      message: `t27 Traceability MCP Server started`,
      project: PROJECT_ROOT,
      enforceL1: ENFORCE_L1,
    })
  );
}

main().catch((error) => {
  console.error(
    JSON.stringify({
      level: "error",
      message: `Failed to start MCP server: ${error}`,
    })
  );
  process.exit(1);
});
