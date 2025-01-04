import { useEffect, useReducer, useState } from "react";
import SearchHeadBar from "./SearchHeadBar";
import SearchLabelsBar from "./SearchLabelsBar";
import { Channel, invoke, SERIALIZE_TO_IPC_FN } from "@tauri-apps/api/core";
import AsyncLock from "async-lock";
import SearchDisplay from "./SearchDisplay";

export enum TextSearchMethod {
  Contains = "contains",
  Regex = "regex",
  Fuzzy = "fuzzy",
}

export interface SearchConstraintStruct {
  search_text: String;
  include_label: Set<String>;
  exclude_label: Set<String>;
  text_search_method: TextSearchMethod;
}

function search_constraint_struct_2_list(
  search_constraint_struct: SearchConstraintStruct
): SearchConstraint[] {
  let constraints: SearchConstraint[] = [];

  if (search_constraint_struct.search_text !== "") {
    switch (search_constraint_struct.text_search_method) {
      case TextSearchMethod.Contains:
        constraints.push({
          type: "textContains",
          data: search_constraint_struct.search_text,
        });
        break;
      case TextSearchMethod.Regex:
        constraints.push({
          type: "textRegex",
          data: search_constraint_struct.search_text,
        });
        break;
      case TextSearchMethod.Fuzzy:
        constraints.push({
          type: "textFuzzy",
          data: search_constraint_struct.search_text,
        });
        break;
    }
  }

  for (let label of search_constraint_struct.include_label) {
    constraints.push({
      type: "hasLabel",
      data: label,
    });
  }

  for (let label of search_constraint_struct.exclude_label) {
    constraints.push({
      type: "notHasLabel",
      data: label,
    });
  }

  return constraints;
}

function search_constraint_struct_reducer(
  state: SearchConstraintStruct,
  action: {
    type:
      | "set_search_text"
      | "insert_include_label"
      | "remove_include_label"
      | "insert_exclude_label"
      | "remove_exclude_label"
      | "set_text_search_method";
    value: String;
  }
): SearchConstraintStruct {
  switch (action.type) {
    case "set_search_text":
      return { ...state, search_text: action.value };
    case "insert_include_label":
      state.include_label.add(action.value);
      return { ...state, include_label: state.include_label };
    case "remove_include_label":
      state.include_label.delete(action.value);
      return { ...state, include_label: state.include_label };
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

/// use lock to ensure only one insert operation is running
let SEARCH_LOCK = new AsyncLock();
interface SearchResult {
  clips: Map<number, Clip[]>;
}

export const SEARCH_RESULT: SearchResult = {
  clips: new Map<number, Clip[]>(),
};

function search_result_insert(
  set_rebuild_num: React.Dispatch<React.SetStateAction<number>>,
  data: {
    clip: Clip;
    sessionId: number;
  }
) {
  SEARCH_LOCK.acquire("search_result_insert", async () => {
    // get the keys of the SEARCH_RESULT
    let keys = Array.from(SEARCH_RESULT.clips.keys());
    // sort the keys
    keys.sort();
    // get the last key
    let last_key = keys[keys.length - 1];
    if (last_key === undefined || data.sessionId > last_key) {
      // if the data is new, insert it
      SEARCH_RESULT.clips.set(data.sessionId, [data.clip]);

      // clean up the old data
      for (let key of keys) {
        SEARCH_RESULT.clips.delete(key);
      }
    } else if (data.sessionId < last_key) {
      // do nothing if the data is outdated
      return;
    } else if (data.sessionId === last_key) {
      // if the data is the same session, append it
      SEARCH_RESULT.clips.get(data.sessionId)?.push(data.clip);
    }

    // add rebuild num to the current unix timestamp after insert to force rebuild
    set_rebuild_num(new Date().getTime());
  });
}

let SEARCH_SESSION_ID: number = 0;

enum ClipType {
  Text = "text",
  Image = "image",
  File = "file",
  Html = "html",
  Rtf = "rtf",
}

interface Clip {
  /// The text of the clip.
  /// After the clip is created, the text should not be changed
  data: String;
  /// The search text of the clip
  searchText: String;
  /// The type of the clip
  clipType: ClipType;
  /// in seconds
  timestamp: number;
  /// the id of the clip
  id: number;
  /// the labels of the clip
  /// each label is a string
  labels: String[];
}

type SearchConstraint =
  /// Search for the text that contains the given text
  | {
      type: "textContains";

      data: String;
    }
  /// Search for the text that match the regex
  | {
      type: "textRegex";
      data: String;
    }
  /// Search for the text that match the fuzzy search
  | {
      type: "textFuzzy";
      data: String;
    }
  /// Timestamp that is greater than the given timestamp
  | {
      type: "timestampGreaterThan";
      data: number;
    }
  /// Timestamp that is less than the given timestamp
  | {
      type: "timestampLessThan";
      data: number;
    }
  /// Has the given label
  | {
      type: "hasLabel";
      data: String;
    }
  /// Does not have the given label
  | {
      type: "notHasLabel";
      data: String;
    }
  /// Limit the number of results
  | {
      type: "limit";
      data: number;
    };

function search_invoke(
  session_id: number,
  set_rebuild_num: React.Dispatch<React.SetStateAction<number>>,
  search_constraint_struct: SearchConstraintStruct,
  additional_constraints: SearchConstraint[] = []
) {
  const onEvent = new Channel<{
    clip: Clip;
    sessionId: number;
  }>();
  onEvent.onmessage = (message) => {
    search_result_insert(set_rebuild_num, message);
  };

  let constraints: SearchConstraint[] = [
    {
      type: "limit",
      data: 30,
    },
  ];

  constraints = constraints.concat(additional_constraints);
  constraints = constraints.concat(search_constraint_struct_2_list(search_constraint_struct));

  invoke("search_clips", {
    onEvent,
    constraints,
    sessionId: session_id,
  });
}

export default function Search() {
  const [search_constraint_struct, set_search_constraint_struct] = useReducer(
    search_constraint_struct_reducer,
    {
      search_text: "",
      include_label: new Set<String>(),
      exclude_label: new Set<String>(),
      text_search_method: TextSearchMethod.Contains,
    }
  );

  const [rebuild_num, set_rebuild_num] = useState<number>(0);

  useEffect(() => {
    SEARCH_SESSION_ID += 1;
    const session_id = SEARCH_SESSION_ID;
    search_invoke(session_id, set_rebuild_num, search_constraint_struct);
  }, [search_constraint_struct]);

  return (
    <div className="h-screen w-screen">
      <SearchHeadBar
        set_search_constraint_struct={set_search_constraint_struct}
        search_constraint_struct={search_constraint_struct}
      />
      <SearchLabelsBar
        set_search_constraint_struct={set_search_constraint_struct}
        search_constraint_struct={search_constraint_struct}
      />
      <SearchDisplay rebuild_num={rebuild_num} />
    </div>
  );
}
