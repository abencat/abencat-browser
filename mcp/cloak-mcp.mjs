#!/usr/bin/env node
/**
 * Cloak Fingerprint Browser — MCP server (stdio, JSON-RPC 2.0, zero dependencies).
 *
 * Exposes the local automation HTTP API (cloak-headless / desktop app) as MCP
 * tools so AI agents (Claude Desktop, Claude Code, Cursor, etc.) can list,
 * create, start and stop fingerprint-browser environments and get the DevTools
 * WebSocket endpoint to drive them with Puppeteer/Playwright/Selenium.
 *
 * Config (env):
 *   CLOAK_API_BASE   default http://127.0.0.1:50327
 *   CLOAK_API_TOKEN  required for create/delete/start/stop (read from the
 *                    service log: `journalctl -u cloak-headless | grep token`)
 *
 * Requires Node 18+ (global fetch). Run: node cloak-mcp.mjs
 */

const API_BASE = (process.env.CLOAK_API_BASE || "http://127.0.0.1:50327").replace(/\/$/, "");
const API_TOKEN = process.env.CLOAK_API_TOKEN || "";
const SERVER_INFO = { name: "cloak-fingerprint-browser", version: "1.0.0" };
const PROTOCOL_VERSION = "2024-11-05";

async function apiGet(path, params = {}) {
  const url = new URL(API_BASE + path);
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== "") url.searchParams.set(k, String(v));
  }
  const res = await fetch(url, { method: "GET" });
  const text = await res.text();
  let json;
  try { json = JSON.parse(text); } catch { json = { raw: text }; }
  if (!res.ok) {
    const msg = json && json.msg ? json.msg : `HTTP ${res.status}`;
    throw new Error(`${path}: ${msg}`);
  }
  return json;
}

const withToken = (p = {}) => ({ ...p, token: API_TOKEN });

const TOOLS = [
  {
    name: "list_profiles",
    description: "List all fingerprint-browser environments (id, name, project, running).",
    inputSchema: { type: "object", properties: {} },
    handler: () => apiGet("/api/v1/profiles"),
  },
  {
    name: "status",
    description: "Get controller status (version, headless mode, currently running ids).",
    inputSchema: { type: "object", properties: {} },
    handler: () => apiGet("/api/v1/status"),
  },
  {
    name: "create_profile",
    description: "Create a new fingerprint environment. Returns its id.",
    inputSchema: {
      type: "object",
      properties: {
        name: { type: "string", description: "Environment name (optional)." },
        projectId: { type: "string", description: "Target project id (optional)." },
      },
    },
    handler: (a) => apiGet("/api/v1/profile/create", withToken({ name: a.name, projectId: a.projectId })),
  },
  {
    name: "delete_profile",
    description: "Delete a fingerprint environment by id (stops it first).",
    inputSchema: {
      type: "object",
      properties: { id: { type: "string", description: "Environment id." } },
      required: ["id"],
    },
    handler: (a) => apiGet("/api/v1/profile/delete", withToken({ id: a.id })),
  },
  {
    name: "start_browser",
    description:
      "Launch a fingerprint browser for an environment. Returns ws (CDP WebSocket), debuggerAddress and debugPort. Connect Puppeteer/Playwright with `ws`, or Selenium with `debuggerAddress`.",
    inputSchema: {
      type: "object",
      properties: {
        id: { type: "string", description: "Environment id." },
        url: { type: "string", description: "Optional URL to open on launch." },
        headless: { type: "boolean", description: "Override headless mode for this launch." },
      },
      required: ["id"],
    },
    handler: (a) =>
      apiGet("/api/v1/browser/start", withToken({
        id: a.id,
        url: a.url,
        headless: a.headless === undefined ? undefined : a.headless ? 1 : 0,
      })),
  },
  {
    name: "stop_browser",
    description: "Stop the running browser for an environment by id.",
    inputSchema: {
      type: "object",
      properties: { id: { type: "string", description: "Environment id." } },
      required: ["id"],
    },
    handler: (a) => apiGet("/api/v1/browser/stop", withToken({ id: a.id })),
  },
  {
    name: "active_endpoints",
    description: "List all currently running environments and their CDP endpoints.",
    inputSchema: { type: "object", properties: {} },
    handler: () => apiGet("/api/v1/browser/active"),
  },
];

const TOOL_MAP = Object.fromEntries(TOOLS.map((t) => [t.name, t]));

function send(msg) {
  process.stdout.write(JSON.stringify(msg) + "\n");
}

function reply(id, result) {
  send({ jsonrpc: "2.0", id, result });
}

function replyError(id, code, message) {
  send({ jsonrpc: "2.0", id, error: { code, message } });
}

async function handleMessage(msg) {
  const { id, method, params } = msg;
  // Notifications have no id and need no response.
  if (id === undefined || id === null) {
    return;
  }
  switch (method) {
    case "initialize":
      return reply(id, {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: { tools: {} },
        serverInfo: SERVER_INFO,
      });
    case "ping":
      return reply(id, {});
    case "tools/list":
      return reply(id, {
        tools: TOOLS.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })),
      });
    case "tools/call": {
      const tool = TOOL_MAP[params && params.name];
      if (!tool) return replyError(id, -32602, `Unknown tool: ${params && params.name}`);
      try {
        const data = await tool.handler((params && params.arguments) || {});
        return reply(id, { content: [{ type: "text", text: JSON.stringify(data, null, 2) }] });
      } catch (err) {
        return reply(id, {
          isError: true,
          content: [{ type: "text", text: String(err && err.message ? err.message : err) }],
        });
      }
    }
    default:
      return replyError(id, -32601, `Method not found: ${method}`);
  }
}

// Newline-delimited JSON-RPC over stdin. Track in-flight async calls so we
// don't exit (on stdin close) before their responses are written.
let buffer = "";
let pending = 0;
let ending = false;
const maybeExit = () => {
  // Defer to the next tick so stdin's handle finishes closing first; calling
  // process.exit() synchronously inside the 'end' handler trips a libuv
  // assertion on Windows. All responses are already flushed by this point.
  if (ending && pending === 0) setImmediate(() => process.exit(0));
};
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;
  let idx;
  while ((idx = buffer.indexOf("\n")) >= 0) {
    const line = buffer.slice(0, idx).trim();
    buffer = buffer.slice(idx + 1);
    if (!line) continue;
    let msg;
    try { msg = JSON.parse(line); } catch { continue; }
    pending++;
    Promise.resolve(handleMessage(msg))
      .catch((e) => process.stderr.write(`mcp error: ${e}\n`))
      .finally(() => { pending--; maybeExit(); });
  }
});
process.stdin.on("end", () => { ending = true; maybeExit(); });
