import { type } from "os"
import { useEffect } from "react"

interface IntInputBoxProps {
  label: string
  onChange: (value: number) => void
  getValue: () => Promise<number>
  defaultValue: number
}

const IntInputBoxTemplate: React.FC<IntInputBoxProps> = (props: IntInputBoxProps) => {
  const { label, onChange, getValue, defaultValue } = props
  useEffect(() => {
    async function fetchData() {
      const res = await getValue()
      onChange(res)
    }
    void fetchData()
  }, [getValue, onChange])
  return (
    <div className="flex flex-col">
      <label htmlFor="int-input-box" className="text-xl">
        {label}
      </label>
      <input
        id="int-input-box"
        type="number"
        className="border border-gray-200 rounded-md p-2"
        onChange={(e) => props.onChange(parseInt(e.target.value, 10))}
        defaultValue={defaultValue}
      />
    </div>
  )
}

export { IntInputBoxTemplate }
