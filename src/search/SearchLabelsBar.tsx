import { useEffect, useState } from "react";
import { SearchConstraintStruct } from "./Search";
import { t } from "../i18n";
import { invoke } from "@tauri-apps/api/core";

function SearchLabelsBarButton(props: {
  label: string;
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
  const [
    click_to_display_clip_does_not_have_this_label_title,
    set_click_to_display_clip_does_not_have_this_label_title,
  ] = useState("search.click_to_display_clip_does_not_have_this_label");
  useEffect(() => {
    t("search.click_to_display_clip_does_not_have_this_label").then((res) => {
      set_click_to_display_clip_does_not_have_this_label_title(res);
    });
  }, []); // run once
  const [
    click_to_display_clip_have_this_label_title,
    set_click_to_display_clip_have_this_label_title,
  ] = useState("search.click_to_display_clip_have_this_label");
  useEffect(() => {
    t("search.click_to_display_clip_have_this_label").then((res) => {
      set_click_to_display_clip_have_this_label_title(res);
    });
  }, []); // run once
  const [
    click_to_remove_label_constraint_title,
    set_click_to_remove_label_constraint_title,
  ] = useState("search.click_to_remove_label_constraint");
  useEffect(() => {
    t("search.click_to_remove_label_constraint").then((res) => {
      set_click_to_remove_label_constraint_title(res);
    });
  }, []); // run once

  console.log(props.search_constraint_struct);

  if (
    props.search_constraint_struct.neutral_label.size !== 0 &&
    props.search_constraint_struct.neutral_label.has(props.label)
  ) {
    return (
      <button
        className="py-1 px-2"
        title={click_to_display_clip_does_not_have_this_label_title}
        onClick={() => {
          props.set_search_constraint_struct({
            type: "remove_neutral_label",
            value: props.label,
          });
          props.set_search_constraint_struct({
            type: "insert_exclude_label",
            value: props.label,
          });
        }}
      >
        {props.label}
      </button>
    );
  } else if (
    props.search_constraint_struct.exclude_label.size !== 0 &&
    props.search_constraint_struct.exclude_label.has(props.label)
  ) {
    return (
      <button
        className="py-1 px-2 bg-red-200 dark:bg-red-600"
        title={click_to_display_clip_have_this_label_title}
        onClick={() => {
          props.set_search_constraint_struct({
            type: "remove_exclude_label",
            value: props.label,
          });
        }}
      >
        {props.label}
      </button>
    );
  } else {
    return (
      <button
        className="py-1 px-2 bg-green-200 dark:bg-green-600"
        title={click_to_remove_label_constraint_title}
        onClick={() => {
          props.set_search_constraint_struct({
            type: "insert_neutral_label",
            value: props.label,
          });
        }}
      >
        {props.label}
      </button>
    );
  }
}

export default function SearchLabelsBar(props: {
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
  // get all labels from the frontend
  const [labels, set_labels] = useState<string[]>([]);
  useEffect(() => {
    invoke<string[]>("get_all_labels", {}).then((res) => {
      set_labels(res);
    });
  }, []); // run once

  // a flex bar to display all labels with gray background color with a line separator
  return (
    <div className="flex flex-nowrap overflow-x-auto bg-gray-200 dark:bg-gray-600 text-black dark:text-white divide-x-2 divide-gray-600 dark:divide-gray-400 no-scrollbar">
      <SearchLabelsBarButton
        label="favorite"
        search_constraint_struct={props.search_constraint_struct}
        set_search_constraint_struct={props.set_search_constraint_struct}
      />
      <SearchLabelsBarButton
        label="pinned"
        search_constraint_struct={props.search_constraint_struct}
        set_search_constraint_struct={props.set_search_constraint_struct}
      />
      {labels.map((label) => {
        return (
          <SearchLabelsBarButton
            label={label}
            search_constraint_struct={props.search_constraint_struct}
            set_search_constraint_struct={props.set_search_constraint_struct}
          />
        );
      })}
    </div>
  );
}
