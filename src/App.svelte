<script lang="ts">
  import autoAnimate from "@formkit/auto-animate";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";

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
    <button
      type="button"
      class="rounded bg-pink-700 hover:bg-pink-800 transition active:scale-90 px-1"
      on:click={cleanTodos}>&#x1F5D1;</button
    >
    <input class="rounded px-1" name="todo" />
    <button
      class="rounded-tr-lg rounded bg-green-500 hover:bg-green-600 transition active:scale-90 px-1"
      type="submit">&#x1F4BE;</button
    >
  </form>

  <ul class="overflow-y-auto container px-5" use:autoAnimate>
    {#each todos as todo, idx (todo.id)}
      <article
        class="grid grid-cols-[max-content_auto_min-content] gap-5 my-2 items-center"
      >
        <div id="done-toggle" class="inline-flex items-center">
          <label
            class="relative flex cursor-pointer items-center rounded-full p-3"
            for="checkbox"
          >
            <input
              type="radio"
              class="before:content[''] peer relative h-5 w-5 cursor-pointer appearance-none rounded-md border border-blue-gray-200 transition-all before:absolute before:top-2/4 before:left-2/4 before:block before:h-12 before:w-12 before:-translate-y-2/4 before:-translate-x-2/4 before:rounded-full before:bg-blue-gray-500 before:opacity-0 before:transition-opacity checked:border-pink-500 checked:bg-pink-500 checked:before:bg-pink-500 hover:before:opacity-10"
              id="checkbox"
              checked={todo.done}
              on:click={() => removeTodo(todo.id)}
            />
            <div
              class="pointer-events-none absolute top-2/4 left-2/4 -translate-y-2/4 -translate-x-2/4 text-white opacity-0 transition-opacity peer-checked:opacity-100"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-3.5 w-3.5"
                viewBox="0 0 20 20"
                fill="currentColor"
                stroke="currentColor"
                stroke-width="1"
              >
                <path
                  fill-rule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clip-rule="evenodd"
                />
              </svg>
            </div>
          </label>
        </div>

        <p
          class="font-semibold hover:text-white break-words {todo.done
            ? 'line-through'
            : ''}"
        >
          {todo.message}
        </p>
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
