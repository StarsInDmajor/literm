import { writable } from 'svelte/store';

interface TerminalSession {
  id: string;
  terminal: any;
  socket: WebSocket;
  isShared: boolean;
}

function createTerminalRegistry() {
  const { subscribe, set, update } = writable<Map<string, TerminalSession>>(new Map());

  return {
    subscribe,

    // Register a new terminal session
    register: (paneId: string, terminal: any, socket: WebSocket) => {
      const sessionId = `term-${paneId}`;
      const session: TerminalSession = {
        id: sessionId,
        terminal,
        socket,
        isShared: false
      };

      update(sessions => {
        const newSessions = new Map(sessions);
        newSessions.set(sessionId, session);
        return newSessions;
      });

      return sessionId;
    },

    // Get a terminal session by pane ID
    getTerminal: (paneId: string) => {
      const sessionId = `term-${paneId}`;
      const sessions = new Map();
      // Note: In a real implementation, we'd access the current store value
      // For now, return null to indicate no sharing
      return null;
    },

    // Share an existing terminal session
    shareTerminal: (fromPaneId: string, toPaneId: string) => {
      const sessionId = `term-${fromPaneId}`;
      update(sessions => {
        const newSessions = new Map(sessions);
        const session = newSessions.get(sessionId);
        if (session) {
          session.isShared = true;
        }
        return newSessions;
      });
    }
  };
}

export const terminalRegistry = createTerminalRegistry();