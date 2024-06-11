import { writable } from "svelte/store";

type Toast = {
  id: number;
  message: string;
  type?: "success" | "error" | "info";
  dismissible?: boolean;
  timeout?: number;
};

export const toasts = writable<Toast[]>([]);

export const addToast = (_toast: Omit<Toast, "id">) => {
  // Create a unique ID so we can easily find/remove it
  // if it is dismissible/has a timeout.
  const id = Math.floor(Math.random() * 10000);

  // Setup some sensible defaults for a toast.
  const defaults = {
    id,
    type: "info",
    dismissible: true,
    timeout: 5000,
  } as const;

  let toast = { ...defaults, ..._toast };

  // Push the toast to the top of the list of toasts
  toasts.update((all) => [toast, ...all]);

  // If toast is dismissible, dismiss it after "timeout" amount of time.
  if (toast.timeout) setTimeout(() => dismissToast(id), toast.timeout);
};

export const dismissToast = (id: number) => {
  toasts.update((all) => all.filter((t) => t.id !== id));
};
