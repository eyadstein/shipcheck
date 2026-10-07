import { useCallback, useEffect, useState } from "react";

export type AsyncState<T> =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "ready"; data: T };

export type Loadable<T> = AsyncState<T> & { reload: () => void };

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : "Something went wrong";
}

/** Runs `load` whenever `deps` change, cancelling the previous request. */
export function useAsync<T>(
  load: (signal: AbortSignal) => Promise<T>,
  deps: readonly unknown[],
): Loadable<T> {
  const [state, setState] = useState<AsyncState<T>>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    setState({ status: "loading" });
    load(controller.signal).then(
      (data) => {
        if (!controller.signal.aborted) setState({ status: "ready", data });
      },
      (error: unknown) => {
        if (!controller.signal.aborted) setState({ status: "error", message: messageOf(error) });
      },
    );
    return () => controller.abort();
  }, [...deps, attempt]);

  const reload = useCallback(() => setAttempt((count) => count + 1), []);
  return { ...state, reload };
}
