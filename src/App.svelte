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
    <input class="rounded px-1" name="todo" />
    <button
      class="rounded-tr-lg rounded bg-green-500 hover:bg-green-600 transition active:scale-90 px-1"
      type="submit">&#x1F4BE;</button
    >
  </form>

  <ul class="overflow-y-auto container px-5">
    {#each todos as todo, idx (todo.id)}
      <article
        class="grid grid-cols-[max-content_auto_min-content] gap-5 my-2 items-start"
      >
        <button
          class="rounded bg-red-700 hover:bg-red-800 transition active:scale-90 px-2"
          on:click={() => removeTodo(todo.id)}>&times;</button
        >
        <p class="font-semibold hover:text-white break-words">{todo.message}</p>
        <div class="flex gap-2 w-[70px] border-solid border-1 border-white">
          {#if idx > 0}
            <button
              class="rounded bg-gray-500 hover:bg-gray-600 transition active:scale-90 px-2 mr-auto"
              on:click={() => moveUp(todo.id)}>&#9650;</button
            >
          {/if}
          {#if idx < todos.length - 1}
            <button
              class="rounded bg-gray-500 hover:bg-gray-600 transition active:scale-90 px-2 ml-auto"
              on:click={() => moveDown(todo.id)}>&#9660;</button
            >
          {/if}
        </div>
      </article>
    {/each}
  </ul>
</main>
