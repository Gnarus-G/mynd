<script lang="ts">
  type Todo = {
    id: string;
    message: string;
    done?: boolean;
  };

  export let todo: Todo;
  export let disableMoveUp = false;
  export let disableMoveDown = false;

  export let onCheck: (id: string) => void;
  export let onMoveUp: (id: string) => void;
  export let onMoveDown: (id: string) => void;

  let overDropzone = false;

  export let onDropBelow: (draggedTodoId: string, belowTodoId: string) => void;
</script>

<article
  class="grid grid-cols-[max-content_auto_min-content] gap-5 mt-2 items-center"
  draggable="true"
  on:dragstart={(ev) => {
    console.log("[TodoItem] started drag on todo id:", todo.id);
    ev.dataTransfer.dropEffect = "move";
    ev.dataTransfer.setData("application/todo-id", todo.id);
    ev.dataTransfer.setData("text/plain", todo.message);
  }}
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
        on:click={() => onCheck(todo.id)}
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
    {#if !disableMoveUp}
      <button
        class="rounded bg-gray-500 hover:bg-gray-600 transition active:scale-90 px-2 mr-auto"
        on:click={() => onMoveUp(todo.id)}>&#9650;</button
      >
    {/if}
    {#if !disableMoveDown}
      <button
        class="rounded bg-gray-500 hover:bg-gray-600 transition active:scale-90 px-2 ml-auto"
        on:click={() => onMoveDown(todo.id)}>&#9660;</button
      >
    {/if}
  </div>
</article>
<div
  class="dropzone ml-14 bg-slate-700 rounded-lg"
  aria-label="drop zone below a todo item"
  data-dropready={overDropzone}
  on:dragover={(ev) => {
    overDropzone = true;
    ev.preventDefault(); // necessary to enable drop handler
    console.log("[TodoItem] dragged over dropzone of todo id:", todo.id);
    ev.dataTransfer.dropEffect = "move";
  }}
  on:dragleave={(_) => {
    overDropzone = false;
    console.log("[TodoItem] leaving dropzone of todo id:", todo.id);
  }}
  on:drop={(ev) => {
    overDropzone = false;
    ev.preventDefault();
    const fromId = ev.dataTransfer.getData("application/todo-id");
    const message = ev.dataTransfer.getData("text/plain");
    console.log("[TodoItem] dropped below todo id:", todo.id, {
      fromId,
      message,
    });

    onDropBelow(fromId, todo.id);
  }}
/>

<style>
  .dropzone {
    transition:
      transform 250ms 125ms ease-in-out,
      opacity 500ms ease-out;
    opacity: 0;
    height: 0.5rem;
  }

  .dropzone[data-dropready="true"] {
    opacity: 1;
    transform: scaleY(2);
  }
</style>
