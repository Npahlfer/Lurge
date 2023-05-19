export default function(el: Element) {
  const h = el.clientHeight
  const paddingTopBottom = 16
  const style = window.getComputedStyle(el)
  return h + parseInt(style.marginTop, 10) + parseInt(style.marginBottom, 10) + paddingTopBottom
}
