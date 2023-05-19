<script lang="ts">
import { invoke } from '@tauri-apps/api/tauri'
import debounce from '../utils/debounce'
import updateAppSize from '../utils/updateAppSize'
import type { ResType } from '../types'

let search = ''
let port = ''
let ports: number[] = []
let placeholder = 'Port nr. to kill'

async function onSearch() {
  if (search.length === 0) ports = []
  if (search.length < 2) return

  const response: ResType<Number[]> = await invoke('handle_search', {
    search,
  })
  ports = Array.from(response.result)
}

let debouncedOnSearch = debounce(onSearch, 700)

function handleSearch(event: Event) {
  const input = event.target as HTMLInputElement
  search = input.value

  debouncedOnSearch()
}

async function handleKill(event: SubmitEvent) {
  event.preventDefault()

  await invoke('proc_kill', { port })

  placeholder = `Killed port ${port}`
  port = ''

  setTimeout(() => {
    placeholder = 'Port nr. to kill'
  }, 1500)
}

function handlePortClick(event: MouseEvent) {
  const button = event.target as HTMLButtonElement
  port = button.value
}

updateAppSize()
</script>

<div class="root">
  <input
    class="input"
    type="search"
    placeholder="Search by name, eg. 'node'"
    on:input="{handleSearch}"
    bind:value="{search}" />

  {#if ports.length}
    <div class="searchResultArea">
      <h4>Open ports:</h4>

      <div class="searchSuggestions">
        {#each ports as port}
          <button class="searchSuggestion" value="{port}" on:click="{handlePortClick}">
            {port}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <form class="formArea" on:submit="{handleKill}">
    <input class="input" placeholder="{placeholder}" bind:value="{port}" />

    <button class="submitBtn">Kill</button>
  </form>
</div>

<style>
.root {
  display: flex;
  flex-direction: column;
}

.root > * + * {
  margin-top: var(--spacing);
}

:is(.input, .formArea) {
  width: 100%;
}

.formArea {
  display: flex;
  gap: var(--spacing);
}

.searchSuggestions {
  max-height: 150px;
  padding: 4px var(--spacing);
  border-radius: var(--border-radius);
  background-color: var(--bg-paper-color);

  display: flex;
  flex-direction: column;
  overflow: auto;
}

.searchSuggestions > * + * {
  margin-top: calc(var(--spacing) / 2);
  border-top: 1px solid var(--divider-color);
}

.searchSuggestion {
  min-width: 100px;
  box-shadow: none;
  border-radius: 0;
}
</style>
