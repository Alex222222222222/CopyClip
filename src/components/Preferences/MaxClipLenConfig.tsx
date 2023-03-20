import { invoke } from "@tauri-apps/api"

import { IntInputBoxTemplate } from "./ConfigTemplate"

const MaxClipLenConfig: React.FC = () => {
  const defaultValue = 50
  const label = "Max Clip Length"
  const onChange = (value: number) => {
    if (value !== null && value !== undefined && value > 0) {
      invoke("set_max_clip_len", { data: value }).catch((reason) => {
        console.log("Failed to invoke Rust command 'set_max_clip_len'")
        console.log(reason)
      })
    }
  }
  // eslint-disable-next-line @typescript-eslint/require-await
  const getValue = async () => {
    invoke("get_max_clip_len")
      .then((value) => {
        // try parse value to number
        const res = parseInt(value as string, 10)
        return res
      })
      .catch((reason) => {
        console.log("Failed to invoke Rust command 'get_max_clip_len'")
        console.log(reason)
        return defaultValue
      })
    return defaultValue
  }

  return (
    <IntInputBoxTemplate
      label={label}
      onChange={onChange}
      getValue={getValue}
      defaultValue={defaultValue}
    />
  )
}

export { MaxClipLenConfig }
