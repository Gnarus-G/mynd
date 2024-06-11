<script lang="ts">
  import autoAnimate from "@formkit/auto-animate";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import TodoItem from "../lib/TodoItem.svelte";
  import Toasts from "$lib/toasts/Toasts.svelte";
  import { erroneous } from "$lib/utils";
  import { addToast } from "$lib/toasts/store";

  type Todo = {
    id: string;
    message: string;
    created_at: string;
    done?: boolean;
  };

  let todos: Todo[] = [];

  onMount(load);

  function handleError(e: string) {
    addToast({
      type: "error",
      message: e,
    });
    console.error("[error] %s", e);
  }

  async function load() {
    await erroneous<Todo[]>(invoke("load"))({
      success: (data) => {
        todos = data;
        console.log("[page] loaded todos", todos);
      },
      error: handleError,
    });
  }

  async function addTodo(item: string) {
    if (!item) return;

    await erroneous<Todo[]>(
      invoke("add", {
        todo: item,
      }),
    )({
      success: (data) => {
        todos = data;
      },
      error: handleError,
    });
  }

  async function removeTodo(id: string) {
    await erroneous<Todo[]>(invoke("remove", { id }))({
      success: (data) => (todos = data),
      error: handleError,
    });
  }

  async function cleanTodos() {
    await erroneous<Todo[]>(invoke("remove_done"))({
      success: (data) => (todos = data),
      error: handleError,
    });
  }

  async function moveUp(id: string) {
    await erroneous<Todo[]>(invoke("move_up", { id }))({
      success: (data) => (todos = data),
      error: handleError,
    });
  }

  async function moveDown(id: string) {
    await erroneous<Todo[]>(invoke("move_down", { id }))({
      success: (data) => (todos = data),
      error: handleError,
    });
  }

  async function moveBelow(sourceTodoId: string, targetTodoId: string) {
    await erroneous<Todo[]>(
      invoke("move_below", {
        id: sourceTodoId,
        targetId: targetTodoId,
      }),
    )({
      success: (data) => (todos = data),
      error: handleError,
    });
  }
</script>

<svelte:window on:focus={load} />

<main
  class="h-screen overflow-y-auto bg-gray-800 text-gray-300 [&_input]:bg-gray-700 flex flex-col items-center justify-center gap-10"
>
  <form
    class="sticky top-4"
    on:submit|preventDefault={(e) => {
      const el = e.currentTarget.elements.namedItem("todo");
      // @ts-ignore
      addTodo(el.value);
      // @ts-ignore
      el.value = "";
    }}
  >
    <button
      type="button"
      class="rounded bg-pink-800 hover:bg-pink-900 transition active:scale-90 px-1 shadow-md shadow-slate-800 hover:shadow-sm"
      on:click={cleanTodos}>&#x1F5D1;</button
    >
    <input class="rounded px-1" name="todo" />
    <button
      class="rounded-tr-lg rounded bg-slate-400 hover:bg-slate-500 transition active:scale-90 px-1 shadow-md shadow-slate-800 hover:shadow-sm"
      type="submit">&#x1F4BE;</button
    >
  </form>

  <ul class="overflow-y-auto container px-5" use:autoAnimate>
    {#each todos as todo, idx (todo.id)}
      <TodoItem
        {todo}
        onCheck={removeTodo}
        onMoveUp={moveUp}
        onMoveDown={moveDown}
        disableMoveUp={idx === 0}
        disableMoveDown={idx === todos.length - 1}
        onDropBelow={moveBelow}
      />
    {/each}
  </ul>
</main>

<Toasts />
