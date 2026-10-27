import { createContext, useContext, type ReactNode } from "react";

const FeedbackContext = createContext<((message: string) => void) | null>(null);

export function FeedbackProvider({
  children,
  notify,
}: {
  children: ReactNode;
  notify: (message: string) => void;
}) {
  return (
    <FeedbackContext.Provider value={notify}>
      {children}
    </FeedbackContext.Provider>
  );
}

export function useFeedback() {
  const notify = useContext(FeedbackContext);
  if (!notify) throw new Error("FeedbackProvider is required");
  return notify;
}
