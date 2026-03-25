import useSWR from "swr";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/tauri-env";

interface TrafficData {
  up: number;
  down: number;
  is_fresh?: boolean;
}

interface UseTrafficDataOptions {
  enabled?: boolean;
}

const trafficFetcher = async (): Promise<TrafficData> => {
  if (!isTauri()) {
    return { up: 0, down: 0, is_fresh: false };
  }
  return await invoke<TrafficData>("get_traffic");
};

export const useTrafficData = (options: UseTrafficDataOptions = {}) => {
  const { enabled = true } = options;

  const { data, error, isLoading, mutate } = useSWR<TrafficData>(
    enabled ? "getTraffic" : null,
    trafficFetcher,
    {
      refreshInterval: 1000,
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
