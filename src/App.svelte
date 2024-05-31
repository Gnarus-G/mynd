<script lang="ts">
  import autoAnimate from "@formkit/auto-animate";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import TodoItem from "./TodoItem.svelte";

  type Todo = {
    id: string;
    message: string;
    done?: boolean;
  };

  let todos: Todo[] = [];

  onMount(load);

  async function load() {
    todos = await invoke("load");
  }

  async function addTodo(item: string) {
    if (!item) return;
    todos = await invoke("add", {
      todo: item,
    });
  }

  async function removeTodo(id: string) {
    todos = await invoke("remove", { id });
  }

  async function cleanTodos() {
    todos = await invoke("remove_done");
  }

  async function moveUp(id: string) {
    todos = await invoke("move_up", { id });
  }

  async function moveDown(id: string) {
    todos = await invoke("move_down", { id });
  }

  async function moveBelow(sourceTodoId: string, targetTodoId: string) {
    todos = await invoke("move_below", {
      id: sourceTodoId,
      targetId: targetTodoId,
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
