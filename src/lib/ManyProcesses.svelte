<script lang="ts">
import { onMount } from 'svelte'
import { invoke } from '@tauri-apps/api/tauri'
import type { ResType, ProcessManyInfo } from '../types'

let processes: ProcessManyInfo[] = []
let minLength = 10
let maxLength = 50

function getMaxText(n) {
  return n >= maxLength ? '> ' : ''
}

onMount(async () => {
  async function getProcesses() {
    const response: ResType<ProcessManyInfo[]> = await invoke('get_many_processes', {
      minLength,
      maxLength,
    })
    processes = response.result
  }
  getProcesses()

  const interval = setInterval(getProcesses, 10000)

  return () => clearInterval(interval)
})
</script>

<div class="root">
  <div class="header"><span class="name">Name</span><span>Count</span></div>
  <ul class="body">
    {#each processes as process}
      <li class="item">
        <span class="name">{process.name}</span><span
          >{getMaxText(process.count)}{process.count}</span>
      </li>
    {/each}
  </ul>
</div>

<style>
.root {
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  /* TODO feels weird to have this margin on the column headers
     but it feels weird to not have it as well */
  margin: 0 var(--spacing);
}

.header > * {
  font-size: 11px;
  font-weight: bold;
  flex: 0 1 40%;
}

.body {
  margin: 0;
  padding: 4px var(--spacing);
  border-radius: var(--border-radius);
  background-color: var(--bg-paper-color);
  list-style: none;
  font-size: 12px;
}

.body > * + * {
  border-top: 1px solid var(--divider-color);
}

.item {
  display: flex;
}

.item > * {
  flex: 0 1 40%;
}

.name {
  flex: 1 1 60%;
}

.name + * {
  margin-left: var(--spacing);
}
</style>
