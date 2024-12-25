import React, { useEffect, useState } from "react";
import { SearchConstraintStruct, TextSearchMethod } from "./Search";
import { SlMagnifierAdd } from "react-icons/sl";
import { VscRegex } from "react-icons/vsc";
import { PiMagnifyingGlassDuotone } from "react-icons/pi";
import { t } from "../i18n";
import { LuMoonStar } from "react-icons/lu";
import { FaRegSun } from "react-icons/fa";
import { PiGearFineFill } from "react-icons/pi";
import { FiPaperclip } from "react-icons/fi";

function EnableFuzzySearchIcon(props: {
  search_constraint_struct: SearchConstraintStruct;
  set_search_constraint_struct: React.ActionDispatch<
    [
      action: {
        type:
          | "set_search_text"
          | "insert_neutral_label"
          | "remove_neutral_label"
          | "insert_exclude_label"
          | "remove_exclude_label"
          | "set_text_search_method";
        value: String;
      }
    ]
  >;
}) {
  const [fuzzy_title, set_fuzzy_title] = useState("search.enable_fuzzy_search");
  useEffect(
    () => {
      t("search.enable_fuzzy_search").then((res) => {
        set_fuzzy_title(res);
      });
    },
    [] // run once
  );

  function fuzzy_on_click() {
    if (
      props.search_constraint_struct.text_search_method ===
      TextSearchMethod.Fuzzy
    ) {
      props.set_search_constraint_struct({
        type: "set_text_search_method",
        value: TextSearchMethod.Contains,
      });
    } else {
      props.set_search_constraint_struct({
        type: "set_text_search_method",
        value: TextSearchMethod.Fuzzy,
      });
    }
  }

  return (
    <>
      {props.search_constraint_struct.text_search_method ===
      TextSearchMethod.Fuzzy ? (
        <SlMagnifierAdd
          className="p-1 my-1 mr-2 h-6 w-6 hover:cursor-pointer rounded-lg bg-blue-400"
          title={fuzzy_title}
          onClick={fuzzy_on_click}
        />
      ) : (
        <SlMagnifierAdd
          className="p-1 my-1 mr-2 h-6 w-6 hover:cursor-pointer"
          title={fuzzy_title}
          onClick={fuzzy_on_click}
        />
      )}
    </>
  );
}

function EnableRegexSearchIcon(props: {
  search_constraint_struct: SearchConstraintStruct;
  set_search_constraint_struct: React.ActionDispatch<
    [
      action: {
        type:
          | "set_search_text"
          | "insert_neutral_label"
          | "remove_neutral_label"
          | "insert_exclude_label"
          | "remove_exclude_label"
          | "set_text_search_method";
        value: String;
      }
    ]
  >;
}) {
  const [regex_title, set_regex_title] = useState("search.enable_regex_search");
  useEffect(
    () => {
      t("search.enable_regex_search").then((res) => {
        set_regex_title(res);
      });
    },
    [] // run once
  );

  function regex_on_click() {
    if (
      props.search_constraint_struct.text_search_method ===
      TextSearchMethod.Regex
    ) {
      props.set_search_constraint_struct({
        type: "set_text_search_method",
        value: TextSearchMethod.Contains,
      });
    } else {
      props.set_search_constraint_struct({
        type: "set_text_search_method",
        value: TextSearchMethod.Regex,
      });
    }
  }

  return (
    <>
      {props.search_constraint_struct.text_search_method ===
      TextSearchMethod.Regex ? (
        <VscRegex
          className="p-1 my-1 ml-1 h-6 w-6 hover:cursor-pointer rounded-lg bg-blue-400"
          title={regex_title}
          onClick={regex_on_click}
        />
      ) : (
        <VscRegex
          className="p-1 my-1 ml-1 h-6 w-6 hover:cursor-pointer"
          title={regex_title}
          onClick={regex_on_click}
        />
      )}
    </>
  );
}

function SearchInputBox(props: {
  search_constraint_struct: SearchConstraintStruct;
  set_search_constraint_struct: React.ActionDispatch<
    [
      action: {
        type:
          | "set_search_text"
          | "insert_neutral_label"
          | "remove_neutral_label"
          | "insert_exclude_label"
          | "remove_exclude_label"
          | "set_text_search_method";
        value: String;
      }
    ]
  >;
}) {
  // add event listener for keydown "/" to focus on search bar
  useEffect(() => {
    function keydown_handler(event: KeyboardEvent) {
      if (event.key === "/") {
        document.getElementById("search-bar-input")?.focus();
      }
    }
    document.addEventListener("keydown", keydown_handler);
    return () => {
      document.removeEventListener("keydown", keydown_handler);
    };
  }, []); // run once

  const [placeholder, set_placeholder] = useState("search.search_placeholder");
  useEffect(() => {
    t("search.search_placeholder").then((res) => {
      set_placeholder(res);
    });
  }, []); // run once

  function text_box_change_callback(
    event: React.ChangeEvent<HTMLInputElement>
  ) {
    const target = event.target as HTMLInputElement;
    props.set_search_constraint_struct({
      type: "set_search_text",
      value: target.value,
    });
  }

  return (
    <input
      className="bg-transparent text-black dark:text-white flex-grow border-none"
      placeholder={placeholder}
      id="search-bar-input"
      onChange={text_box_change_callback}
    />
  );
}

function ToggleDarkModeIcon(props: {
  is_dark: boolean;
  set_is_dark: React.Dispatch<React.SetStateAction<boolean>>;
}) {
  const [dark_mode_title, set_dark_mode_title] = useState(
    "search.toggle_dark_mode"
  );
  useEffect(() => {
    t("search.toggle_dark_mode").then((res) => {
      set_dark_mode_title(res);
    });
  }, []); // run once

  function toggle_dark_mode() {
    const value = !props.is_dark;

    props.set_is_dark(value);

    if (value) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  return (
    <span
      className="h-12 mx-2 hover:cursor-pointer"
      onClick={toggle_dark_mode}
      title={dark_mode_title}
    >
      {props.is_dark ? (
        <LuMoonStar className="my-3 h-6 w-6" />
      ) : (
        <FaRegSun className="my-3 h-6 w-6" />
      )}
    </span>
  );
}

function SettingsPageIcon() {
  const [settings_title, set_settings_title] = useState("search.preferences");
  useEffect(() => {
    t("search.preferences").then((res) => {
      set_settings_title(res);
    });
  }, []); // run once

  function handle_settings_page_click() {
    // TODO add preferences page
  }

  return (
    <span className="h-12 mr-2 hover:cursor-pointer">
      <PiGearFineFill
        className="my-3 h-6 w-6"
        title={settings_title}
        onClick={handle_settings_page_click}
      />
    </span>
  );
}

export default function SearchHeadBar(props: {
  search_constraint_struct: SearchConstraintStruct;
  set_search_constraint_struct: React.ActionDispatch<
    [
      action: {
        type:
          | "set_search_text"
          | "insert_neutral_label"
          | "remove_neutral_label"
          | "insert_exclude_label"
          | "remove_exclude_label"
          | "set_text_search_method";
        value: String;
      }
    ]
  >;
}) {
  const [is_dark, set_is_dark] = useState(false);

  return (
    <div className="w-ful flex flex-row h-12">
      <div className="bg-gray-100 dark:bg-gray-900 border rounded-full px-2 m-2 flex flex-row h-8 text-lg w-32">
        <FiPaperclip className="my-1 mr-1 h-6 w-6" />
        <span className="text-black dark:text-white">{"CopyClip"}</span>
      </div>
      <div className="border rounded-full h-8 my-2 bg-gray-100 flex-grow dark:bg-gray-900 flex flex-row text-black dark:text-white">
        <PiMagnifyingGlassDuotone className="m-2 h-4" />
        <SearchInputBox
          search_constraint_struct={props.search_constraint_struct}
          set_search_constraint_struct={props.set_search_constraint_struct}
        />
        <EnableRegexSearchIcon
          search_constraint_struct={props.search_constraint_struct}
          set_search_constraint_struct={props.set_search_constraint_struct}
        />
        <EnableFuzzySearchIcon
          search_constraint_struct={props.search_constraint_struct}
          set_search_constraint_struct={props.set_search_constraint_struct}
        />
      </div>

      <ToggleDarkModeIcon is_dark={is_dark} set_is_dark={set_is_dark} />

      <SettingsPageIcon />
    </div>
  );
}
