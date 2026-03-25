import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/tauri-env";

// 等待 Tauri 环境初始化的辅助函数
const waitForTauri = async (
  maxRetries = 10,
  delayMs = 100,
): Promise<boolean> => {
  for (let i = 0; i < maxRetries; i++) {
    if (isTauri()) {
      return true;
    }
    // 等待一段时间再重试
    await new Promise((resolve) => setTimeout(resolve, delayMs));
  }
  return false;
};

export const copyClashEnv = async () => {
  if (!isTauri()) {
    return;
  }
  return invoke("copy_clash_env");
};

export const getVergeConfig = async () => {
  // 如果在浏览器环境中，直接返回默认值
  if (!isTauri()) {
    return {} as IVergeConfig;
  }

  try {
    return await invoke<IVergeConfig>("get_verge_config");
  } catch (error) {
    return {} as IVergeConfig;
  }
};

export const patchVergeConfig = async (patch: Partial<IVergeConfig>) => {
  if (!isTauri()) {
    return;
  }
  return invoke<void>("patch_verge_config", { patch });
};
