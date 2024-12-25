import { useReducer } from "react";
import SearchHeadBar from "./SearchHeadBar";
import SearchLabelsBar from "./SearchLabelsBar";

export enum TextSearchMethod {
  Contains = "contains",
  Regex = "regex",
  Fuzzy = "fuzzy",
}

export interface SearchConstraintStruct {
  search_text: String;
  neutral_label: Set<String>;
  exclude_label: Set<String>;
  text_search_method: TextSearchMethod;
}

function search_constraint_struct_reducer(
  state: SearchConstraintStruct,
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
): SearchConstraintStruct {
  switch (action.type) {
    case "set_search_text":
      return { ...state, search_text: action.value };
    case "insert_neutral_label":
      state.neutral_label.add(action.value);
      return { ...state, neutral_label: state.neutral_label };
    case "remove_neutral_label":
      state.neutral_label.delete(action.value);
      return { ...state, neutral_label: state.neutral_label };
    case "insert_exclude_label":
      state.exclude_label.add(action.value);
      return { ...state, exclude_label: state.exclude_label };
    case "remove_exclude_label":
      state.exclude_label.delete(action.value);
      return { ...state, exclude_label: state.exclude_label };
    case "set_text_search_method":
      return { ...state, text_search_method: action.value as TextSearchMethod };
  }
}

export default function Search() {
  const [search_constraint_struct, set_search_constraint_struct] = useReducer(
    search_constraint_struct_reducer,
    {
      search_text: "",
      neutral_label: new Set<String>(),
      exclude_label: new Set<String>(),
      text_search_method: TextSearchMethod.Contains,
    }
  );

  return (
    <div className="h-full w-full">
      <SearchHeadBar
        set_search_constraint_struct={set_search_constraint_struct}
        search_constraint_struct={search_constraint_struct}
      />
      <SearchLabelsBar
        set_search_constraint_struct={set_search_constraint_struct}
        search_constraint_struct={search_constraint_struct}
      />
    </div>
  );
}
