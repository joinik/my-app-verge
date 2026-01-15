let vergeConfigCache: IVergeConfig | null | undefined
export const getPreloadConfig = () => vergeConfigCache
export const setPreloadConfig = (config: IVergeConfig | null) => {
  vergeConfigCache = config
}
