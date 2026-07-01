"use client";
import { createContext, useContext, useEffect, useRef, useState, ReactNode } from "react";

type SocketStatus = "connecting" | "connected" | "disconnected";

interface WebSocketContextValue {
  status: SocketStatus;
  send: (data: string) => void;
}

const WebSocketContext = createContext<WebSocketContextValue>({
  status: "disconnected",
  send: () => {},
});

export function WebSocketProvider({ url, children }: { url: string; children: ReactNode }) {
  const wsRef = useRef<WebSocket | null>(null);
  const retryDelay = useRef(1000);
  const [status, setStatus] = useState<SocketStatus>("disconnected");

  useEffect(() => {
    let cancelled = false;

    function connect() {
      if (cancelled) return;
      setStatus("connecting");
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        retryDelay.current = 1000;
        setStatus("connected");
      };

      ws.onmessage = (event) => {
        // Relay messages to the window so child components can listen
        window.postMessage(event.data, "*");
      };

      ws.onclose = () => {
        if (cancelled) return;
        setStatus("disconnected");
        // Reduce reconnection attempts when tab is not visible
        const base = document.visibilityState === "hidden" ? Math.min(retryDelay.current * 2, 30000) : retryDelay.current;
        const delay = Math.min(base, 30000);
        retryDelay.current = delay * 2;
        setTimeout(connect, delay);
      };
    }

    // Pause reconnects when tab is hidden, resume when visible
    function handleVisibilityChange() {
      if (document.visibilityState === "visible" && !wsRef.current || wsRef.current?.readyState === WebSocket.CLOSED) {
        retryDelay.current = 1000;
        connect();
      }
    }

    document.addEventListener("visibilitychange", handleVisibilityChange);
    connect();

    return () => {
      cancelled = true;
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      wsRef.current?.close();
    };
  }, [url]);

  const send = (data: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) wsRef.current.send(data);
  };

  return (
    <WebSocketContext.Provider value={{ status, send }}>
      {status === "disconnected" && (
        <div
          role="alert"
          aria-live="polite"
          style={{ background: "#f59e0b", color: "#fff", padding: "4px 12px", fontSize: 12 }}
        >
          Reconnecting...
        </div>
      )}
      {children}
    </WebSocketContext.Provider>
  );
}

export const useSocket = () => useContext(WebSocketContext);
