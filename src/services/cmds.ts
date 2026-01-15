import { invoke } from '@tauri-apps/api/core'

export const copyClashEnv = async () => {
  return invoke('copy_clash_env')
}
export const getVergeConfig = async () =>
  invoke<IVergeConfig>('get_verge_config')

export const patchVergeConfig = async (patch: Partial<IVergeConfig>) =>
  invoke<void>('patch_verge_config', { patch })
