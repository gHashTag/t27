import { useCallback, useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";

import {
  createSession,
  deleteSession,
  fetchSessions,
  createSandboxToken,
} from "./lib/api";
import { ChatApp } from "./components/ChatApp";

type Session = {
  id: string;
  name: string;
  status: string;
  railwayServiceId: string;
  createdAt: string;
  updatedAt: string;
};

type RefreshOptions = {
  showSyncIndicator?: boolean;
};

const areSessionsEqual = (next: Session[], prev: Session[]) => {
  if (next.length !== prev.length) return false;

  for (let index = 0; index < next.length; index += 1) {
    const nextSession = next[index];
    const prevSession = prev[index];

    if (
      nextSession.id !== prevSession.id ||
      nextSession.updatedAt !== prevSession.updatedAt ||
      nextSession.status !== prevSession.status ||
      nextSession.name !== prevSession.name ||
      nextSession.railwayServiceId !== prevSession.railwayServiceId
    ) {
      return false;
    }
  }

  return true;
};

function App() {
  // Must be before useState calls - env var
  const isLocalMode = import.meta.env.VITE_LOCAL_MODE === "true";

  // Tab state for navigation - default to chat in local mode
  const [activeTab, setActiveTab] = useState<"sessions" | "chat">(
    isLocalMode ? "chat" : "sessions"
  );

  // Direct access: skip authentication - use dummy token
  const [token] = useState<string>("direct-access");
  const [sessions, setSessions] = useState<Session[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [creatingCount, setCreatingCount] = useState(0);
  const [deletingSessionIds, setDeletingSessionIds] = useState<string[]>([]);
  const [launchingSessionId, setLaunchingSessionId] = useState<string | null>(
    null,
  );
  const [syncing, setSyncing] = useState(false);
  const [lastSyncedAt, setLastSyncedAt] = useState<Date | null>(null);
  const [showDeleted, setShowDeleted] = useState(false);

  // Session dashboard with direct access (v2 - in-memory storage)
  const handleApiError = (message: string) => {
    setError(message);
  };

  const refreshSessions = useCallback(async (options: RefreshOptions = {}) => {
    if (!token) return;
    const { showSyncIndicator = true } = options;
    if (showSyncIndicator) {
      setSyncing(true);
    }
    try {
      const data = await fetchSessions(token);
      setSessions((currentSessions) =>
        areSessionsEqual(data, currentSessions) ? currentSessions : data,
      );
      setLastSyncedAt(new Date());
    } catch (error) {
      if (error instanceof Error) {
        if (error.message === "unauthorized") {
          handleApiError("Session expired. Please refresh the page.");
          return;
        }
        handleApiError(error.message);
      }
    } finally {
      if (showSyncIndicator) {
        setSyncing(false);
      }
    }
  }, [token]);

  useEffect(() => {
    // Skip session fetching in local mode - no backend required
    if (token && !isLocalMode) {
      refreshSessions();
    }
  }, [refreshSessions, token, isLocalMode]);

  useEffect(() => {
    // Skip polling in local mode
    if (!token || isLocalMode) return;
    const interval = setInterval(() => {
      refreshSessions({ showSyncIndicator: false });
    }, 2000);

    return () => clearInterval(interval);
  }, [refreshSessions, token, isLocalMode]);

  const handleCreateSession = async (
    event: FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    if (isLocalMode) return;
    if (!token) return;
    setCreatingCount((count) => count + 1);
    setError(null);
    try {
      await createSession(token, name.trim() ? name.trim() : undefined);
      setName("");
      await refreshSessions();
    } catch (error) {
      if (error instanceof Error) {
        handleApiError(error.message);
      }
    } finally {
      setCreatingCount((count) => Math.max(0, count - 1));
    }
  };

  const handleDeleteSession = async (id: string) => {
    if (!token) return;
    setDeletingSessionIds((current) =>
      current.includes(id) ? current : [...current, id],
    );
    setError(null);
    try {
      await deleteSession(token, id);
      await refreshSessions();
    } catch (error) {
      if (error instanceof Error) {
        handleApiError(error.message);
      }
    } finally {
      setDeletingSessionIds((current) => current.filter((item) => item !== id));
    }
  };

  const handleLaunchSession = async (session: Session) => {
    if (!token) return;
    setLaunchingSessionId(session.id);
    setError(null);
    try {
      const sandboxToken = await createSandboxToken(token, session.id);
      // Explicit URL construction to avoid env var issues
      const sandboxUrl = `${window.location.origin}/sandbox?token=${encodeURIComponent(sandboxToken.token)}`;
      window.open(sandboxUrl, '_blank');
    } catch (error) {
      if (error instanceof Error) {
        handleApiError(error.message);
      }
    } finally {
      setLaunchingSessionId(null);
    }
  };

  const visibleSessions = useMemo(
    () =>
      showDeleted
        ? sessions
        : sessions.filter((session) => session.status !== "deleted"),
    [sessions, showDeleted],
  );

  const stats = useMemo(() => {
    const active = visibleSessions.filter(
      (session) => session.status === "active",
    );
    return {
      total: visibleSessions.length,
      active: active.length,
    };
  }, [visibleSessions]);

  return (
    <div className="app">
      {/* Tab Navigation */}
      <nav className="tabs">
        <button
          className={`tab ${activeTab === "sessions" ? "active" : ""}`}
          onClick={() => setActiveTab("sessions")}
          aria-selected={activeTab === "sessions"}
          role="tab"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="9" y1="21" x2="9" y2="9" />
          </svg>
          Sessions
        </button>
        <button
          className={`tab ${activeTab === "chat" ? "active" : ""}`}
          onClick={() => setActiveTab("chat")}
          aria-selected={activeTab === "chat"}
          role="tab"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
          Chat
        </button>
      </nav>

      <main className="panel" style={{ display: activeTab === "sessions" ? "block" : "none" }}>
          <div className="panel-body">
            <div className="panel-header">
              <div>
                <h2>Session dashboard</h2>
                <p>
                  Live Railway sessions.
                  <span className={`sync-indicator ${syncing ? "is-active" : ""}`}>
                    Syncing...
                  </span>
                </p>
              </div>
              <div className="panel-actions">
                <div className="metric">
                  <span>Total</span>
                  <strong>{stats.total}</strong>
                </div>
              <div className="metric">
                <span>Active</span>
                <strong>{stats.active}</strong>
              </div>
              <button
                className="ghost"
                onClick={() => setShowDeleted((prev) => !prev)}
              >
                {showDeleted ? "Hide deleted" : "Show deleted"}
              </button>
            </div>
            </div>

            <form className="form inline" onSubmit={handleCreateSession}>
              <label className="field">
                <span>New session</span>
                <input
                  type="text"
                  placeholder="Sandbox name (optional)"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  disabled={isLocalMode}
                />
              </label>
              <button
                className="primary"
                type="submit"
                disabled={isLocalMode}
              >
                {creatingCount > 0 ? "Creating..." : "Create session"}
              </button>
              <button
                className="ghost"
                type="button"
                onClick={() => refreshSessions()}
                disabled={syncing}
              >
                Refresh
              </button>
            </form>

            {isLocalMode ? (
              <div className="alert">
                Session creation is disabled in local mode.
              </div>
            ) : null}

            {error ? <div className="alert">{error}</div> : null}

            <div className="sessions">
              {visibleSessions.length === 0 ? (
                <div className="empty">
                  <h3>No sessions to show</h3>
                  <p>
                    {showDeleted
                      ? "Create your first sandbox to start testing."
                      : "Toggle deleted sessions to view retired sandboxes."}
                  </p>
                </div>
              ) : (
                visibleSessions.map((session) => (
                  <article className="session-card" key={session.id}>
                    <div>
                      <h3>{session.name}</h3>
                      <p className="muted">
                        <span
                          className={`status-tag status-${session.status}`}
                        >
                          {session.status}
                        </span>
                        <span> · </span>
                        {new Date(session.createdAt).toLocaleString()}
                      </p>
                      <p className="id">Service {session.railwayServiceId}</p>
                    </div>
                    <div className="session-actions">
                      <button
                        className="ghost"
                        onClick={() => handleLaunchSession(session)}
                        disabled={
                          session.status !== "active" ||
                          launchingSessionId === session.id
                        }
                      >
                        {launchingSessionId === session.id
                          ? "Launching..."
                          : "Open sandbox"}
                      </button>
                      <button
                        className="danger"
                        onClick={() => handleDeleteSession(session.id)}
                        disabled={
                          session.status === "deleted" ||
                          deletingSessionIds.includes(session.id)
                        }
                      >
                        Retire
                      </button>
                    </div>
                  </article>
                ))
              )}
            </div>

            <div className="footnote">
              {lastSyncedAt
                ? `Last synced ${lastSyncedAt.toLocaleTimeString()}`
                : "Awaiting first sync."}
            </div>
          </div>
        </main>

      {/* Chat App */}
      <div style={{ display: activeTab === "chat" ? "flex" : "none", flexDirection: "column" }}>
        <ChatApp />
      </div>
    </div>
  );
}

export default App;
