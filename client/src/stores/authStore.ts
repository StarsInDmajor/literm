import { writable, derived } from 'svelte/store';

export interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

const initialState: AuthState = {
  isAuthenticated: false,
  isLoading: false,
  error: null,
};

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>(initialState);

  return {
    subscribe,
    set,
    update,
    // Check if user is authenticated by validating session
    checkAuth: async () => {
      update(state => ({ ...state, isLoading: true }));

      try {
        const response = await fetch('/api/auth/status', {
          credentials: 'include',
        });

        if (response.ok) {
          const data = await response.json();
          update(state => ({
            ...state,
            isAuthenticated: data.authenticated,
            isLoading: false,
            error: null,
          }));
        } else {
          update(state => ({
            ...state,
            isAuthenticated: false,
            isLoading: false,
            error: 'Failed to verify authentication status',
          }));
        }
      } catch (error) {
        console.error('Auth check failed:', error);
        update(state => ({
          ...state,
          isAuthenticated: false,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Authentication check failed',
        }));
      }
    },

    // Login with password
    login: async (password: string) => {
      update(state => ({ ...state, isLoading: true, error: null }));

      try {
        const response = await fetch('/api/login', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          credentials: 'include',
          body: JSON.stringify({ password }),
        });

        if (response.ok) {
          const data = await response.json();
          if (data.ok) {
            update(state => ({
              ...state,
              isAuthenticated: true,
              isLoading: false,
              error: null,
            }));
            return true;
          }
        }

        throw new Error('Login failed');
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Login failed';
        update(state => ({
          ...state,
          isAuthenticated: false,
          isLoading: false,
          error: errorMessage,
        }));
        return false;
      }
    },

    // Logout
    logout: async () => {
      try {
        await fetch('/api/logout', {
          method: 'POST',
          credentials: 'include',
        });
      } catch (error) {
        console.error('Logout error:', error);
      }

      update(state => ({
        ...state,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      }));
    },

    // Clear error
    clearError: () => {
      update(state => ({ ...state, error: null }));
    },
  };
}

export const authStore = createAuthStore();

// Derived stores for convenience
export const isAuthenticated = derived(authStore, state => state.isAuthenticated);
export const authLoading = derived(authStore, state => state.isLoading);
export const authError = derived(authStore, state => state.error);