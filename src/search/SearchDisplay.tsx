import { SEARCH_RESULT } from "./Search";

export default function SearchDisplay(props: { rebuild_num: number }) {
  // get the largest key of SEARCH_RESULT
  let keys = Array.from(SEARCH_RESULT.clips.keys());
  keys.sort();
  let last_key = keys[keys.length - 1];

  return (
    <>
      <div hidden={true}>{props.rebuild_num}</div>
      {last_key === undefined ? (
        <div className="text-center text-2xl text-gray-500 dark:text-gray-400">
          No search result
        </div>
      ) : SEARCH_RESULT.clips.get(last_key)?.length === 0 ? (
        <div className="text-center text-2xl text-gray-500 dark:text-gray-400">
          No search result
        </div>
      ) : (
        SEARCH_RESULT.clips.get(last_key)?.map((clip) => {
          return (
            <div key={clip.id} className="p-2">
              <div className="text-lg font-semibold">{clip.searchText}</div>
            </div>
          );
        })
      )}
    </>
  );
}
