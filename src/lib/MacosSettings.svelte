<script lang="ts">
import { onMount, afterUpdate } from 'svelte'
import { invoke } from '@tauri-apps/api/tauri'
import debounce from '../utils/debounce'
import updateAppSize from '../utils/updateAppSize'
import type { ResType } from '../types'
import Checkbox from './Checkbox.svelte'

let error = ''
let screenshotDir = ''
let screenshotFormat = ''
let createDesktop = true
let showHardDrives = true
let supportedFormats = ['png', 'jpg', 'tif', 'pdf', 'bmp', 'gif']

async function onScreenshotDirChange(dir: string) {
  await invoke('set_screenshot_directory', { dir })
}

async function onCreateDesktopChange(state: boolean) {
  invoke('set_desktop_show', { state })
}

async function onHardDrivesChange(state: boolean) {
  invoke('set_desktop_hard_drives_show', { state })
}

let debouncedScreenshotDirChange = debounce(onScreenshotDirChange, 6000)
let debouncedCreateDesktopChange = debounce(onCreateDesktopChange, 2000)
let debouncedHardDrivesChange = debounce(onHardDrivesChange, 2000)

async function handleScreenshotDirChange(event: Event) {
  const input = event.target as HTMLInputElement

  if (input.value.length < 4) return
  screenshotDir = input.value
  debouncedScreenshotDirChange(input.value)
}

const handleCreateDesktopChange = (event: Event) => {
  const checkbox = event.target as HTMLInputElement
  createDesktop = checkbox.checked
  debouncedCreateDesktopChange(checkbox.checked)
}

const handleHardDrivesChange = (event: Event) => {
  const checkbox = event.target as HTMLInputElement
  showHardDrives = checkbox.checked
  debouncedHardDrivesChange(checkbox.checked)
}

const handleScreenshotFormatChange = (event: Event) => {
  const select = event.target as HTMLSelectElement
  screenshotFormat = select.value
  invoke('set_screenshot_format', { format: select.value })
}

onMount(async () => {
  const hardDrivesShowRes: ResType<boolean> = await invoke('get_desktop_hard_drives_show')
  const desktopShowRes: ResType<boolean> = await invoke('get_desktop_show')
  const screenshotFormatRes: ResType<string> = await invoke('get_screenshot_format')
  const screenshotDirRes: ResType<string> = await invoke('get_screenshot_directory')

  showHardDrives = hardDrivesShowRes.result
  createDesktop = desktopShowRes.result
  screenshotFormat = screenshotFormatRes.result
  screenshotDir = screenshotDirRes.result
})

afterUpdate(() => {
  if (error) {
    setTimeout(() => {
      error = ''
    }, 3000)
  }
})

updateAppSize()
</script>

<form class="root">
  <label for="macos-screenshotDir">
    Set screenshot directory
    <input
      class="input"
      placeholder="Directory"
      id="macos-screenshotDir"
      bind:value="{screenshotDir}"
      on:input="{handleScreenshotDirChange}" />
  </label>

  <label for="macos-screenshotFormat">
    Select screenshot format
    <select
      class="input"
      placeholder="Select format"
      id="macos-screenshotFormat"
      bind:value="{screenshotFormat}"
      on:input="{handleScreenshotFormatChange}">
      {#each supportedFormats as format}
        <option value="{format}">{format}</option>
      {/each}
    </select>
  </label>

  <fieldset class="fieldset">
    <label class="checkboxLabel" for="macos-createDesktop">
      <Checkbox
        type="checkbox"
        id="macos-createDesktop"
        bind:checked="{createDesktop}"
        on:input="{handleCreateDesktopChange}" />
      Show desktop icons
    </label>

    {#if createDesktop}
      <label class="checkboxLabel" for="macos-showHardDrives">
        <Checkbox
          type="checkbox"
          id="macos-showHardDrives"
          bind:checked="{showHardDrives}"
          on:input="{handleHardDrivesChange}" />
        Show desktop hard drives
      </label>
    {/if}
  </fieldset>

  {#if error}
    <span class="error">{error}</span>
  {/if}
</form>

<style>
.root {
  display: flex;
  flex-direction: column;
}

.root > * + * {
  margin-top: var(--spacing);
}

.fieldset {
  display: flex;
  flex-direction: column;
}

.input {
  width: 100%;
}

.checkboxLabel {
  display: flex;
  align-items: center;
  gap: var(--spacing);
}
</style>
