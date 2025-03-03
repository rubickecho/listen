import { useTokenStore } from "../store/tokenStore";
import type { PriceUpdate } from "../types/price";

export function setupWebSocket() {
  const updateTokenData = useTokenStore.getState().updateTokenData;

  const ws = new WebSocket("wss://api.listen-rs.com/v1/adapter/ws");

  ws.onmessage = (event) => {
    try {
      const data: PriceUpdate = JSON.parse(event.data);
      const whitelist = ["AI Rig Complex", 'arc', 'ARC'];
      if (whitelist.includes(data.name)) {
        console.log("Received price update:", data);
      }
      updateTokenData(data);
    } catch (error) {
      console.error("Error parsing message:", error);
    }
  };

  ws.onopen = () => {
    ws.send(
      JSON.stringify({
        action: "subscribe",
        mints: ["*"],
      })
    );
    console.log("WebSocket connection opened");
  };

  ws.onerror = (error) => {
    console.error("WebSocket failed:", error);
  };

  ws.onclose = () => {
    console.log("WebSocket connection closed");
    // Optionally implement reconnection logic here
  };

  return ws;
}
