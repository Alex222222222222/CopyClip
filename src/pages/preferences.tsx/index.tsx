import { NextPage } from "next"

import AppHead from "@/components/AppHead"
import { ClipPerPageConfig } from "@/components/Preferences/ClipPerPageConfig"

const Preferences: NextPage = () => {
  return (
    <div className="flex min-h-screen flex-col bg-white">
      <AppHead></AppHead>
      <h1 className="m-0 text-center text-6xl">Preferences</h1>
      <div className="mx-5 my-2">
        <ClipPerPageConfig></ClipPerPageConfig>
      </div>
    </div>
  )
}

export default Preferences
