import { writable } from "svelte/store";

export type Todo = {
  id: string;
  message: string;
  created_at: string;
  done?: boolean;
};

export const todos = writable<Todo[]>([]);
export const loading = writable(true);
export const mutating = writable(false);
export const apiError = writable<string | null>(null);

export async function load() {
  loading.set(true);
  await request<Todo[]>("/api/todos")
    .then(todos.set)
    .catch(reportError)
    .finally(() => loading.set(false));
}

export async function addTodo(item: string) {
  if (!item.trim()) return;
  return mutate("/api/todos", {
    method: "POST",
    body: JSON.stringify({ message: item }),
  });
}

export async function removeTodo(id: string) {
  return mutate(`/api/todos/${id}/complete`, { method: "POST" });
}

export async function cleanTodos() {
  return mutate("/api/todos/completed", { method: "DELETE" });
}

export async function moveUp(id: string) {
  return mutate(`/api/todos/${id}/move-up`, { method: "POST" });
}

export async function moveDown(id: string) {
  return mutate(`/api/todos/${id}/move-down`, { method: "POST" });
}

export async function moveBelow(sourceTodoId: string, targetTodoId: string) {
  return mutate(`/api/todos/${sourceTodoId}/move-below`, {
    method: "POST",
    body: JSON.stringify({ target_id: targetTodoId }),
  });
}

export async function deleteTodo(id: string) {
  return mutate(`/api/todos/${id}`, { method: "DELETE" });
}

export async function getWebUrl() {
  const config = await request<{ web_url?: string }>("/api/config");
  return config.web_url ?? null;
}

async function mutate(path: string, init: RequestInit) {
  mutating.set(true);
  apiError.set(null);
  try {
    todos.set(await request<Todo[]>(path, init));
  } catch (error) {
    reportError(error);
    throw error;
  } finally {
    mutating.set(false);
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: init.body ? { "content-type": "application/json", ...init.headers } : init.headers,
  });
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.error ?? `Mynd server returned ${response.status}`);
  }
  return response.json();
}

function reportError(error: unknown) {
  apiError.set(error instanceof Error ? error.message : "Could not reach the Mynd server");
}
