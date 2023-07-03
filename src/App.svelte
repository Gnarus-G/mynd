<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";

  type Todo = {
    id: string;
    message: string;
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

  async function moveUp(id: string) {
    todos = await invoke("move_up", { id });
  }

  async function moveDown(id: string) {
    todos = await invoke("move_down", { id });
  }
</script>

<main class="container">
  <form
    on:submit|preventDefault={(e) => {
      const el = e.currentTarget.elements.namedItem("todo");
      // @ts-ignore
      addTodo(el.value);
      // @ts-ignore
      el.value = "";
    }}
  >
    <input name="todo" />
    <button type="submit">[+]></button>
  </form>

  {#each todos as todo, idx (todo.id)}
    <article style="display: flex; align-items: center; gap: 10px;">
      <button on:click={() => removeTodo(todo.id)}>&times;</button>
      <p>{idx}</p>
      <p>{todo.message}</p>
      <div>
        {#if idx > 0}
          <button on:click={() => moveUp(todo.id)}>&#9650;</button>
        {/if}
        {#if idx < todos.length - 1}
          <button on:click={() => moveDown(todo.id)}>&#9660;</button>
        {/if}
      </div>
    </article>
  {/each}
</main>
