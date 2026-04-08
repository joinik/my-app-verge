import {
  calcuProxies,
  calcuProxyProviders,
  getSystemProxy,
} from "@/services/cmds";
import { SWR_DEFAULTS, SWR_REALTIME } from "@/services/config";
import { useCallback, useMemo } from "react";
import useSWR, { useSWRConfig } from "swr";
import { useSharedSWRPoller } from "./use-shared-ser-poller";
import { getBaseConfig, getRuleProviders, getRules } from "tauri-plugin-mihomo-api";
import { useVerge } from "./use-verge";

export const useProxiesData = () => {
  const { mutate: globalMutate } = useSWRConfig();
  const { data, error, isLoading } = useSWR("getProxies", calcuProxies, {
    ...SWR_REALTIME,
    refreshInterval: 0,
    onError: (err) => console.warn("[AppData] Proxy fetch failed:", err),
  });

  const refreshProxy = useCallback(
    () => globalMutate("getProxies"),
    [globalMutate],
  );

  const pollerRefresh = useCallback(
    () => void globalMutate("getProxies"),
    [globalMutate],
  );
  useSharedSWRPoller("getProxies", SWR_REALTIME.refreshInterval, pollerRefresh);

  return { proxies: data, refreshProxy, isLoading, error };
};

export const useClashConfig = () => {
  const { mutate: globalMutate } = useSWRConfig();
  const { data, error, isLoading } = useSWR("getClashConfig", {
    ...SWR_REALTIME,
    refreshInterval: 0,
  });

  const refreshClashConfig = useCallback(
    () => globalMutate("getClashConfig"),
    [globalMutate],
  );

  const pollerRefresh = useCallback(
    () => void globalMutate("getClashConfig"),
    [globalMutate],
  );
  useSharedSWRPoller(
    "getClashConfig",
    SWR_REALTIME.refreshInterval,
    pollerRefresh,
  );
  return { clashConfig: data, refreshClashConfig, isLoading, error };
};

export const useProxyProvidersData = () => {
  const { data, error, isLoading, mutate } = useSWR(
    "getProxyProviders",
    calcuProxyProviders,
    SWR_DEFAULTS,
  );

  const refreshProxyProviders = useCallback(() => mutate(), [mutate]);

  return {
    proxyProviders: data || {},
    refreshProxyProviders,
    isLoading,
    error,
  };
};

export const useRuleProvidersData = () => {
  const { data, error, isLoading, mutate } = useSWR(
    "getRuleProviders",
    getRuleProviders,
    SWR_DEFAULTS,
  );

  const refreshRuleProviders = useCallback(() => mutate(), [mutate]);

  return {
    ruleProviders: data?.providers || {},
    refreshRuleProviders,
    isLoading,
    error,
  };
};

export const useRulesData = () => {
  const { data, error, isLoading, mutate } = useSWR(
    "getRules",
    getRules,
    SWR_DEFAULTS,
  );

  const refreshRules = useCallback(() => mutate(), [mutate]);

  return {
    rules: data?.rules || [],
    refreshRules,
    isLoading,
    error,
  };
};

export const useSystemProxyData = () => {
  const { data, error, isLoading, mutate } = useSWR(
    "getSystemProxy",
    getSystemProxy,
    SWR_DEFAULTS,
  );

  const refreshSysproxy = useCallback(() => mutate(), [mutate]);

  return {
    sysproxy: data,
    refreshSysproxy,
    isLoading,
    error,
  };
};

// 定义 ClashConfig 类型
// 使用 Awaited 和 ReturnType 工具类型，从 getBaseConfig 函数推断其异步返回值的类型
// ReturnType<typeof getBaseConfig> 获取函数的返回类型（即 Promise<T>）
// Awaited<...> 提取 Promise 解析后的实际类型 T
type ClashConfig = Awaited<ReturnType<typeof getBaseConfig>>;

type SystemProxy = Awaited<ReturnType<typeof getSystemProxy>>;

interface SystemProxyAddressParams {
  clashConfig?: ClashConfig | null;
  systemProxy?: SystemProxy | null;
}
export const useSystemProxyAddress = ({
  clashConfig,
  systemProxy,
}: SystemProxyAddressParams) => {
  const { verge } = useVerge();
  return useMemo(() => {
    if (!verge || !clashConfig) return "-";
    const isPacMode = verge.proxy_auto_config ?? false;

    if (isPacMode) {
      const proxyHost = verge.proxy_host || "127.0.0.1";
      const proxyPort = verge.verge_mixed_port || clashConfig.mixedPort || 7897;
      return [proxyHost, proxyPort].join(":");
    }

    const systemServer = systemProxy?.server;
    if (systemServer && systemServer !== "-" && !systemServer.startsWith(":")) {
      return systemServer;
    }

    const proxyHost = verge.proxy_host || "127.0.0.1";
    const proxyPort = verge.verge_mixed_port || clashConfig.mixedPort || 7897;
    return [proxyHost, proxyPort].join(":");
  }, [clashConfig, systemProxy, verge]);
};

export const useAppUptime = () => {
  const { data, error, isLoading } = useSWR("getAppUptime", {
    ...SWR_DEFAULTS,
    refreshInterval: 3000,
    errorRetryCount: 1,
  });
  return { uptime: data || 0, isLoading, error };
};

export const useRefreshAll = () => {
  const { mutate } = useSWRConfig();
  return useCallback(() => {
    mutate("getProxies");
    mutate("getClashConfig");
    mutate("getRules");
    mutate("getProxyProviders");
    mutate("getRuleProviders");
    mutate("getSystemProxy");
  }, [mutate]);
};
