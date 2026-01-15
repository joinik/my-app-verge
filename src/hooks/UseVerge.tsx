import useSWR from 'swr'
import { getPreloadConfig, setPreloadConfig } from '../services/preload'
import { getVergeConfig, patchVergeConfig } from '../services/cmds'
export const useVerge = () => {
  const initialVergeConfig = getPreloadConfig()
  const { data: verge, mutate: mutateVerge } = useSWR(
    'getVergeConfig',
    async () => {
      const config = await getVergeConfig()
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
