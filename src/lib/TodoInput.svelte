<script>
  import { addTodo, cleanTodos } from "./store";
</script>

<form
  class="sticky bottom-4 container flex gap-5 px-8 items-center"
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
    class="rounded-full bg-rose-800 hover:bg-rose-900 transition active:scale-90 p-2 shadow-md shadow-slate-800 hover:shadow-sm"
    on:click={cleanTodos}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24px"
      viewBox="0 -960 960 960"
      width="24px"
      fill="#e8eaed"
      ><path
        d="m376-300 104-104 104 104 56-56-104-104 104-104-56-56-104 104-104-104-56 56 104 104-104 104 56 56Zm-96 180q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520Zm-400 0v520-520Z"
      /></svg
    >
  </button>
  <input class="rounded-md px-1 w-full h-full" name="todo" />
  <button
    class="flex gap-2 p-2 rounded-lg bg-slate-950 hover:bg-slate-900 transition active:scale-90 shadow-md shadow-slate-800 hover:shadow-sm"
    type="submit"
    ><strong>Add</strong>
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24px"
      viewBox="0 -960 960 960"
      width="24px"
      fill="#e8eaed"
      ><path
        d="M840-680v480q0 33-23.5 56.5T760-120H200q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h480l160 160Zm-80 34L646-760H200v560h560v-446ZM480-240q50 0 85-35t35-85q0-50-35-85t-85-35q-50 0-85 35t-35 85q0 50 35 85t85 35ZM240-560h360v-160H240v160Zm-40-86v446-560 114Z"
      /></svg
    ></button
  >
</form>
