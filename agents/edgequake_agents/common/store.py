"""SQLite store for the /v1 API: custom agents + the conversation index.

ADK sessions (the messages) live in sessions.db via ADK's SqliteSessionService;
this file only keeps what ADK has no place for: agent specs and conversation
titles for listing/search/rename.
"""

from __future__ import annotations

import json
import sqlite3
import uuid
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = """
CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY, owner_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT NOT NULL,
  icon TEXT NOT NULL, instructions TEXT NOT NULL, model TEXT NOT NULL, tools TEXT NOT NULL,
  starters TEXT NOT NULL, temperature REAL NOT NULL, visibility TEXT NOT NULL, workspace_id TEXT,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY, user_id TEXT NOT NULL, agent_id TEXT NOT NULL, title TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS conversations_user ON conversations(user_id, updated_at);
"""
# Conversations are scoped to the (tenant, workspace) they were created in. Rows from
# before this column existed have NULLs; they are claimed by the first workspace their
# owner lists or opens them from (see claim_legacy), then stay there.
MIGRATIONS = ("ALTER TABLE conversations ADD COLUMN tenant_id TEXT",
              "ALTER TABLE conversations ADD COLUMN workspace_id TEXT")
AGENT_FIELDS = ("name", "description", "icon", "instructions", "model", "tools", "starters",
                "temperature", "visibility", "workspace_id")


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


class Store:
    # ponytail: sync sqlite3 on the event loop, one connection per call; fine for
    # small rows. Move to aiosqlite if this ever shows up in latency.
    def __init__(self, path: Path):
        self.path = path
        with self._db() as db:
            db.executescript(SCHEMA)
            cols = {r[1] for r in db.execute("PRAGMA table_info(conversations)")}
            for sql in MIGRATIONS:
                if sql.split()[-2] not in cols:
                    db.execute(sql)

    @contextmanager
    def _db(self):
        db = sqlite3.connect(self.path)
        db.row_factory = sqlite3.Row
        try:
            with db:  # commit / rollback
                yield db
        finally:
            db.close()

    def _q(self, sql: str, *args) -> list[dict]:
        with self._db() as db:
            return [dict(r) for r in db.execute(sql, args).fetchall()]

    # ---------------------------------------------------------------- agents

    @staticmethod
    def _agent(row: dict) -> dict:
        row.update(kind="custom", tools=json.loads(row["tools"]), starters=json.loads(row["starters"]))
        return row

    def get_agent(self, agent_id: str) -> dict | None:
        rows = self._q("SELECT * FROM agents WHERE id = ?", agent_id)
        return self._agent(rows[0]) if rows else None

    def list_agents(self, user_id: str, workspace_id: str | None) -> list[dict]:
        rows = self._q("SELECT * FROM agents WHERE (owner_id = ? AND visibility = 'private') OR "
                       "(visibility = 'workspace' AND workspace_id = ?) ORDER BY updated_at DESC",
                       user_id, workspace_id or "")
        return [self._agent(r) for r in rows]

    def save_agent(self, fields: dict, owner_id: str | None = None, agent_id: str | None = None) -> dict:
        vals = {k: json.dumps(fields[k]) if k in ("tools", "starters") else fields[k] for k in AGENT_FIELDS}
        with self._db() as db:
            if agent_id is None:
                agent_id, ts = str(uuid.uuid4()), now()
                db.execute(f"INSERT INTO agents (id, owner_id, created_at, updated_at, {', '.join(vals)}) "
                           f"VALUES (?, ?, ?, ?, {', '.join('?' * len(vals))})",
                           (agent_id, owner_id, ts, ts, *vals.values()))
            else:
                db.execute(f"UPDATE agents SET updated_at = ?, {', '.join(f'{k} = ?' for k in vals)} WHERE id = ?",
                           (now(), *vals.values(), agent_id))
        return self.get_agent(agent_id)

    def delete_agent(self, agent_id: str) -> None:
        self._q("DELETE FROM agents WHERE id = ?", agent_id)

    # ---------------------------------------------------------------- conversations

    def create_conversation(self, user_id: str, agent_id: str, tenant_id: str | None = None,
                            workspace_id: str | None = None) -> dict:
        conv = {"id": str(uuid.uuid4()), "user_id": user_id, "agent_id": agent_id, "title": "New chat",
                "created_at": now(), "updated_at": now(), "tenant_id": tenant_id, "workspace_id": workspace_id}
        with self._db() as db:
            db.execute("INSERT INTO conversations (id, user_id, agent_id, title, created_at, updated_at, tenant_id, "
                       "workspace_id) VALUES (:id, :user_id, :agent_id, :title, :created_at, :updated_at, "
                       ":tenant_id, :workspace_id)", conv)
        return conv

    def claim_legacy(self, user_id: str, tenant_id: str | None, workspace_id: str | None) -> None:
        """Pre-scoping conversations (workspace NULL) move to the first workspace their
        owner uses them from. Only the owner's own rows, only when a workspace is known."""
        if workspace_id:
            self._q("UPDATE conversations SET tenant_id = ?, workspace_id = ? WHERE user_id = ? AND workspace_id IS NULL",
                    tenant_id, workspace_id, user_id)

    def get_conversation(self, user_id: str, conv_id: str, tenant_id: str | None = None,
                         workspace_id: str | None = None) -> dict | None:
        self.claim_legacy(user_id, tenant_id, workspace_id)
        rows = self._q("SELECT * FROM conversations WHERE id = ? AND user_id = ? AND workspace_id IS ? "
                       "AND (tenant_id IS ? OR tenant_id IS NULL)", conv_id, user_id, workspace_id, tenant_id)
        return rows[0] if rows else None

    def list_conversations(self, user_id: str, agent_id: str | None, q: str | None, limit: int,
                           tenant_id: str | None = None, workspace_id: str | None = None) -> list[dict]:
        self.claim_legacy(user_id, tenant_id, workspace_id)
        sql = ("SELECT * FROM conversations WHERE user_id = ? AND workspace_id IS ? "
               "AND (tenant_id IS ? OR tenant_id IS NULL)")
        args = [user_id, workspace_id, tenant_id]
        if agent_id:
            sql, args = sql + " AND agent_id = ?", args + [agent_id]
        if q:
            sql, args = sql + " AND lower(title) LIKE ? ESCAPE '\\'", args + [
                "%" + q.lower().replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_") + "%"]
        return self._q(sql + " ORDER BY updated_at DESC LIMIT ?", *args, limit)

    def update_conversation(self, conv_id: str, title: str | None = None) -> None:
        if title is None:
            self._q("UPDATE conversations SET updated_at = ? WHERE id = ?", now(), conv_id)
        else:
            self._q("UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?", title, now(), conv_id)

    def delete_conversation(self, conv_id: str) -> None:
        self._q("DELETE FROM conversations WHERE id = ?", conv_id)
