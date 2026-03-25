import useSWR from "swr";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/tauri-env";

interface MemoryData {
  inuse: number;
  is_fresh?: boolean;
}

const memoryFetcher = async (): Promise<MemoryData> => {
  if (!isTauri()) {
    return { inuse: 0, is_fresh: false };
  }
  return await invoke<MemoryData>("get_memory");
};

export const useMemoryData = () => {
  const { data, error, isLoading, mutate } = useSWR<MemoryData>(
    "getMemory",
    memoryFetcher,
    {
      refreshInterval: 5000,
      revalidateOnFocus: false,
      revalidateOnReconnect: false,
    },
  );

  return {
    response: {
      data: data ?? null,
      error,
      isLoading,
    },
    mutate,
  };
};
