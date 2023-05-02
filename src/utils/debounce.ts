export default <F extends (...args: any[]) => void>(fn: F, delayMs: number = 500) => {
  let timeoutId: ReturnType<typeof setTimeout>
  return (...args: Parameters<F>) => {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
    timeoutId = setTimeout(() => {
      fn(...args)
    }, delayMs)
  }
}
