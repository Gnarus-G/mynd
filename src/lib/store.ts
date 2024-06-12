import { writable } from "svelte/store";
import { erroneous, handleError } from "./utils";
import { invoke } from "@tauri-apps/api/core";
import { addToast } from "./toasts/store";

export const listulelement = writable<HTMLUListElement | null>(null);

export type Todo = {
  id: string;
  message: string;
  created_at: string;
  done?: boolean;
};

export const todos = writable<Todo[]>([]);

export async function load() {
  await erroneous<Todo[]>(invoke("load"))({
    success: (data) => {
      todos.set(data);
      console.log("[page] loaded todos", todos);
    },
    error: handleError,
  });
}

export async function addTodo(item: string) {
  if (!item) return;

  await erroneous<Todo[]>(
    invoke("add", {
      todo: item,
    })
  )({
    success: (data) => {
      todos.set(data);
      listulelement.update((el) => {
        // timeout is to give the list element time to refresh with the new item.
        setTimeout(() => {
          el?.lastElementChild?.scrollIntoView({
            behavior: "smooth",
          });
        }, 70);
        return el;
      });
    },
    error: handleError,
  });
}

export async function removeTodo(id: string) {
  await erroneous<Todo[]>(invoke("remove", { id }))({
    success: (data) => todos.set(data),
    error: handleError,
  });
}

export async function cleanTodos() {
  await erroneous<Todo[]>(invoke("remove_done"))({
    success: (data) => todos.set(data),
    error: handleError,
  });
}

export async function moveUp(id: string) {
  await erroneous<Todo[]>(invoke("move_up", { id }))({
    success: (data) => todos.set(data),
    error: handleError,
  });
}

export async function moveDown(id: string) {
  await erroneous<Todo[]>(invoke("move_down", { id }))({
    success: (data) => todos.set(data),
    error: handleError,
  });
}

export async function moveBelow(sourceTodoId: string, targetTodoId: string) {
  await erroneous<Todo[]>(
    invoke("move_below", {
      id: sourceTodoId,
      targetId: targetTodoId,
    })
  )({
    success: (data) => todos.set(data),
    error: handleError,
  });
}

export async function deleteTodo(id: string) {
  await erroneous<Todo[]>(invoke("delete", { id }))({
    success: (data) => {
      todos.set(data);
      addToast({
        type: "success",
        message: "Todo item sucessfully deleted",
      });
    },
    error: handleError,
  });
}
