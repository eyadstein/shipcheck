import { useMemo, useSyncExternalStore } from "react";
import { parseRoute, type Route } from "./router";

function subscribe(onChange: () => void): () => void {
  window.addEventListener("hashchange", onChange);
  return () => window.removeEventListener("hashchange", onChange);
}

export function useRoute(): Route {
  const hash = useSyncExternalStore(
    subscribe,
    () => window.location.hash,
    () => "",
  );
  return useMemo(() => parseRoute(hash), [hash]);
}
