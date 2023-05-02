<script lang="ts">
export let checked = false
export let name = ''
</script>

<input
  class="checkbox"
  type="checkbox"
  bind:checked="{checked}"
  on:input
  name="{name}"
  {...$$restProps} />

<style>
.checkbox {
  --checkbox-width: 30px;
  --checkbox-toggle-size: 12px;
  --checkbox-bg-color: -apple-system-control-accent; /*#4ed164;*/
  position: relative;
  appearance: none;
  outline: none;
  width: var(--checkbox-width);
  height: calc(var(--checkbox-width) - var(--checkbox-toggle-size));
  background-color: #ffffff;
  border: 1px solid #d9dadc;
  border-radius: var(--checkbox-width);
  box-shadow: inset calc(var(--checkbox-toggle-size) * -1) 0 0 0 #ffffff;
  transition: all 100ms linear, border 0ms linear;
  overflow: hidden;
}

.checkbox:after {
  content: '';
  position: absolute;
  top: 1px;
  left: 1px;
  width: 13px;
  height: 13px;
  background-color: transparent;
  border-radius: 50%;
  box-shadow: 1px 1px 3px rgba(0, 0, 0, 0.2);
}
.checkbox:checked {
  border-color: var(--checkbox-bg-color);
  box-shadow: inset var(--checkbox-toggle-size) 0 0 0 var(--checkbox-bg-color);
}

.checkbox:checked:after {
  left: var(--checkbox-toggle-size);
  box-shadow: -2px 2px 2px rgba(0, 0, 0, 0.05);
}
</style>
