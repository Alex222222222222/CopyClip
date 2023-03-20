import { invoke } from "@tauri-apps/api"

import { IntInputBoxTemplate } from "./ConfigTemplate"

const ClipPerPageConfig: React.FC = () => {
  const defaultValue = 20
  const label = "Clip Per Page"
  const onChange = (value: number) => {
    if (value !== null && value !== undefined && value > 0) {
      invoke("set_per_page_data", { data: value }).catch((reason) => {
        console.log("Failed to invoke Rust command 'set_per_page_data'")
        console.log(reason)
      })
    }
  }
  // eslint-disable-next-line @typescript-eslint/require-await
  const getValue = async () => {
    invoke("get_per_page_data")
      .then((value) => {
        // try parse value to number
        const res = parseInt(value as string, 10)
        return res
      })
      .catch((reason) => {
        console.log("Failed to invoke Rust command 'get_per_page_data'")
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

export { ClipPerPageConfig }
