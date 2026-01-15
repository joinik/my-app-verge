import useSWR from 'swr'
export const useVerge = () => {
  const initialVergeConfig = getPreloadConfig()
  const { data: verge, mutate: mutateVerge } = useSWR(
    'getVergeConfig',
    async () => {
      const config = await getPreloadConfig()
      setPreloadConfig(config)
      return config
    },
    {
      fallbackData: initialVergeConfig ?? undefined,
      revalidateOnMount: !initialVergeConfig,
    }
  )

  const patchVerge = async (patch: Partial<IVergeConfig>) => {
    await patchVergeConfig(patch)
    mutateVerge()
  }
  return {
    verge,
    patchVerge,
    mutateVerge,
  }
}
