<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Shield, Eye, EyeOff, AlertTriangle } from 'lucide-svelte';
  import { fade, scale } from 'svelte/transition';
  import { authStore } from '../../stores/authStore';

  export let isOpen = false;
  export let onSuccess: () => void;
  export let onCancel: () => void;

  const dispatch = createEventDispatcher();

  let password = '';
  let showPassword = false;
  let isLoading = false;
  let error: string | null = null;
  let passwordInput: HTMLInputElement;

  async function handleLogin() {
    if (!password.trim()) {
      error = 'Please enter a password';
      return;
    }

    isLoading = true;
    error = null;

    try {
      const success = await authStore.login(password);

      if (success) {
        dispatch('login-success');
        onSuccess();
        password = '';
        error = null;
      } else {
        error = 'Authentication failed';
      }
    } catch (err) {
      console.error('Login error:', err);
      error = err instanceof Error ? err.message : 'Authentication failed';
    } finally {
      isLoading = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onCancel();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onCancel();
    } else if (e.key === 'Enter' && !isLoading) {
      handleLogin();
    }
  }

  function togglePasswordVisibility() {
    showPassword = !showPassword;
    if (passwordInput) {
      passwordInput.type = showPassword ? 'text' : 'password';
    }
  }

  $: if (isOpen) {
    setTimeout(() => {
      passwordInput?.focus();
    }, 100);
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    on:click={handleBackdropClick}
    on:keydown={handleKeyDown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="login-title"
    transition:fade={{ duration: 200 }}
  >
    <div
      class="w-full max-w-md bg-secondary-bg border border-border-color rounded-lg shadow-xl overflow-hidden"
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-border-color bg-tertiary-bg">
        <div class="flex items-center gap-2">
          <Shield size={20} class="text-accent-color" />
          <h2 id="login-title" class="text-lg font-semibold text-primary-text">Authentication Required</h2>
        </div>
        <button
          class="text-secondary-text hover:text-primary-text transition-colors"
          on:click={onCancel}
          aria-label="Close"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-6">
        <p class="text-sm text-secondary-text mb-6">
          Please enter the password to access LiteTerm. This authentication protects your terminal sessions and file system access.
        </p>

        <!-- Password Input -->
        <div class="space-y-4">
          <div>
            <label for="password" class="block text-sm font-medium text-primary-text mb-2">
              Password
            </label>
            <div class="relative">
              <input
                id="password"
                bind:this={passwordInput}
                type="password"
                bind:value={password}
                class="w-full px-3 py-2 pr-10 bg-primary-bg border border-border-color rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent-color focus:border-accent-color"
                placeholder="Enter your password"
                on:input={() => error = null}
                disabled={isLoading}
              />
              <button
                type="button"
                class="absolute right-2 top-1/2 transform -translate-y-1/2 p-1 text-secondary-text hover:text-primary-text transition-colors disabled:opacity-50"
                on:click={togglePasswordVisibility}
                disabled={isLoading}
                aria-label={showPassword ? 'Hide password' : 'Show password'}
              >
                {#if showPassword}
                  <EyeOff size={16} />
                {:else}
                  <Eye size={16} />
                {/if}
              </button>
            </div>
          </div>

          <!-- Error Display -->
          {#if error}
            <div class="flex items-start gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
              <AlertTriangle size={16} class="text-red-400 mt-0.5 flex-shrink-0" />
              <div class="text-sm text-red-400">{error}</div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-tertiary-bg flex justify-end gap-3">
        <button
          class="px-4 py-2 text-sm bg-secondary-bg hover:bg-primary-bg border border-border-color rounded transition-colors"
          on:click={onCancel}
          disabled={isLoading}
        >
          Cancel
        </button>
        <button
          class="px-4 py-2 text-sm bg-accent-color hover:bg-blue-500 text-white border border-accent-color rounded transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
          on:click={handleLogin}
          disabled={isLoading || !password.trim()}
        >
          {#if isLoading}
            <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Authenticating...
          {:else}
            <Shield size={16} />
            Login
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}