import { useState, useEffect } from "react";

export const useVisibility = () => {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const handleVisibilityChange = () => {
      setVisible(!document.hidden);
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return visible;
};
