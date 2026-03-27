import { getProfiles, patchProfile } from "@/services/cmds";
import useSWR from "swr";
import { debuglog } from "util";

export const useProfiles = () => {
  const {
    data: profiles,
    mutate: mutateProfiles,
    error,
    isValidating,
  } = useSWR("getProfiles", getProfiles, {
    revalidateOnFocus: false,
    revalidateOnReconnect: false,
    dedupingInterval: 500, // 减少去重时间，提高响应性
    errorRetryCount: 3,
    errorRetryInterval: 1000,
    refreshInterval: 0, // 完全由手动控制
    onError: (error) => {
      console.error("[useProfiles] SWR错误:", error);
    },
    onSuccess: (data) => {
      debuglog(
        "[useProfiles] 配置数据更新成功，配置数量:",
        data?.items?.length || 0,
      );
    },
  });

  const patchProfiles = async (
    value: Partial<IProfilesConfig>,
    signal?: AbortSignal,
  ) => {
    try {
      if (signal?.aborted) {
        throw new DOMException("Operation was aborted", "AbortError");
      }
      const success = await patchProfilesConfig(value);

      if (signal?.aborted) {
        throw new DOMException("Operation was aborted", "AbortError");
      }
      await mutateProfiles();
      return success;
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        throw error;
      }
      await mutateProfiles();
      throw error;
    }
  };

  const patchCurrent = async (value: Partial<IProfilesConfig>) => {
    if (profiles?.current) {
      await patchProfile(profiles.current, value);
      if (!value.selected) {
        mutateProfiles();
      }
    }
  };
};
