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
        <div className="flex flex-row w-screen divide-x-4 divide-gray-800 h-[calc(100vh-80px)]">
          <div className="grid-cols-1 overflow-y-auto flex-nowrap w-2/5 text-base divide-y-2 divide-gray-600 h-full">
            {SEARCH_RESULT.clips.get(last_key)?.map((clip) => {
              return (
                <div key={clip.id} className="px-2 line-clamp-3 bg-green-50">
                  {clip.searchText}
                </div>
              );
            })}
            <div
              key={"load_more"}
              onClick={() => {
                // load more
              }}
              className="px-2 line-clamp-3 bg-green-50"
            >
              Load more
            </div>
          </div>
          <div className="bg-green-500"></div>
        </div>
      )}
    </>
  );
}
