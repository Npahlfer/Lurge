import { appWindow, LogicalSize } from '@tauri-apps/api/window'
import { afterUpdate } from 'svelte'
import getElOuterHeight from './getElOuterHeight'

export default function () {
  afterUpdate(() => {
    // TODO use a better selector
    const children = document.querySelector('#app main').children

    const height = Array.from(children).reduce((acc, child) => acc + getElOuterHeight(child), 0)
    appWindow.setSize(new LogicalSize(300, height)).catch((error) => console.error(error))
  })
}
