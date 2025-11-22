import { writable } from 'svelte/store';
import type { Terminal } from 'xterm';

interface TerminalSession {
  id: string;
  terminal: Terminal;
  socket: WebSocket;
  paneId: string; // Currently used by this pane
  isShared: boolean;
}

function createTerminalStore() {
  const { subscribe, set, update } = writable<Map<string, TerminalSession>>(new Map());

  // Internal store value
  let sessions = new Map<string, TerminalSession>();

  return {
    subscribe,
    set: (newSessions: Map<string, TerminalSession>) => {
      sessions = newSessions;
      set(newSessions);
    },
    update,
    // For internal use only
    _getInternalSessions: () => sessions,

    // Get session by ID
    getSession: (id: string) => {
      return sessions.get(id);
    },

    // Create new session
    createSession: (paneId: string, terminal: Terminal, socket: WebSocket) => {
      const sessionId = `session-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      const session: TerminalSession = {
        id: sessionId,
        terminal,
        socket,
        paneId,
        isShared: false
      };

      update(sessions => {
        const newSessions = new Map(sessions);
        newSessions.set(sessionId, session);
        return newSessions;
      });

      return sessionId;
    },

    // Share session with another pane
    shareSession: (sessionId: string, newPaneId: string) => {
      update(sessions => {
        const session = sessions.get(sessionId);
        if (session) {
          session.isShared = true;
        }
        return sessions;
      });
    },

    // Stop sharing session from a pane
    stopSharing: (sessionId: string, paneId: string) => {
      update(sessions => {
        const session = sessions.get(sessionId);
        if (session && session.paneId === paneId) {
          session.isShared = false;
        }
        return sessions;
      });
    },

    // Remove session (when pane is destroyed and not shared)
    removeSession: (sessionId: string) => {
      update(sessions => {
        const session = sessions.get(sessionId);
        if (session && !session.isShared) {
          session.socket?.close();
          session.terminal?.dispose();
        }

        const newSessions = new Map(sessions);
        newSessions.delete(sessionId);
        return newSessions;
      });
    },

    // Get all sessions for debugging
    getAllSessions: () => {
      return Array.from(sessions.values());
    }
  };
}

export const terminalStore = createTerminalStore();