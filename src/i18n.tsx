import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export default function T(props: { t_key: string }) {
  const [t_res, set_t_res] = useState<string>(props.t_key);
  useEffect(() => {
    invoke<string>("i18n_t", { key: props.t_key }).then((res) => {
      set_t_res(res);
    });
  }, [props.t_key]);

  return <>{t_res}</>;
}

export async function t(t_key: string): Promise<string> {
  return await invoke<string>("i18n_t", { key: t_key });
}
