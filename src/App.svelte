<script lang="ts">
import PortKiller from './lib/PortKiller.svelte'
import MacosSettings from './lib/MacosSettings.svelte'
</script>

<main class="container">
  <div class="row">
    <h2>Kill network port</h2>
    <PortKiller />
  </div>

  <div class="row">
    <h2>MacOS settings</h2>
    <MacosSettings />
  </div>
</main>

<style>
.container {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  margin: 0;
  padding: calc(var(--spacing) * 2) calc(var(--spacing) * 2);
  overflow: auto;
}

.container > * + * {
  margin-top: var(--spacing);
}

.row {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 100%;
}
</style>
