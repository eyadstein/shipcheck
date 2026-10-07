import type { ReactNode } from "react";
import type { Loadable } from "../useAsync";

export function Loading({ what }: { what: string }) {
  return (
    <p className="state" role="status">
      {`Loading ${what}...`}
    </p>
  );
}

export function ErrorNotice({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="state state-error" role="alert">
      <p>{message}</p>
      <button type="button" className="button" onClick={onRetry}>
        Try again
      </button>
    </div>
  );
}

interface RemoteProps<T> {
  state: Loadable<T>;
  what: string;
  children: (data: T) => ReactNode;
}

/** Shows a loading or error state until the data is ready. */
export function Remote<T>({ state, what, children }: RemoteProps<T>) {
  if (state.status === "loading") {
    return <Loading what={what} />;
  }
  if (state.status === "error") {
    return <ErrorNotice message={state.message} onRetry={state.reload} />;
  }
  return <>{children(state.data)}</>;
}
