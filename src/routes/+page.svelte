<script lang="ts">
  import autoAnimate from "@formkit/auto-animate";
  import { onMount } from "svelte";
  import TodoItem from "../lib/TodoItem.svelte";
  import Toasts from "$lib/toasts/Toasts.svelte";
  import {
    todos,
    load,
    removeTodo,
    moveUp,
    moveDown,
    moveBelow,
    listulelement,
  } from "$lib/store";
  import TrashCan from "$lib/trash/TrashCan.svelte";
  import TodoInput from "$lib/TodoInput.svelte";

  onMount(async () => {
    await load();
    $listulelement?.lastElementChild?.scrollIntoView({
      behavior: "smooth",
    });
  });
</script>

<svelte:window on:focus={load} />

<main
  class="h-screen overflow-y-auto bg-gray-800 text-gray-300 [&_input]:bg-gray-700 flex flex-col items-center justify-center gap-10"
>
  <ul
    bind:this={$listulelement}
    class="overflow-y-auto container px-5"
    use:autoAnimate
  >
    {#each $todos as todo, idx (todo.id)}
      <TodoItem
        {todo}
        onCheck={removeTodo}
        onMoveUp={moveUp}
        onMoveDown={moveDown}
        disableMoveUp={idx === 0}
        disableMoveDown={idx === $todos.length - 1}
        onDropBelow={moveBelow}
      />
    {/each}
  </ul>

  <TodoInput />
</main>

<TrashCan />

<Toasts />
