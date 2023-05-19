export type ResType<T> = {
  success: boolean
  result: T
  error: string
}

export type ProcessManyInfo = {
  name: String,
  count: Number,
}
