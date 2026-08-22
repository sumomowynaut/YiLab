import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { useAppStore } from "./stores/useAppStore";

function App() {
  const counter = useAppStore((state) => state.counter);
  const increment = useAppStore((state) => state.increment);
  const reset = useAppStore((state) => state.reset);
  const [greetMsg, setGreetMsg] = useState("");

  async function greet() {
    try {
      setGreetMsg(await invoke<string>("greet", { name: "Pika" }));
    } catch (error) {
      setGreetMsg(`invoke 失败：${String(error)}（请在 Tauri 环境中运行）`);
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>PikaXiangqi</CardTitle>
          <CardDescription>中国象棋复盘与分析 — 项目骨架（Phase 0）</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-3">
            <Button onClick={increment}>计数 {counter}</Button>
            <Button variant="outline" onClick={reset}>
              重置
            </Button>
          </div>
          <div className="flex items-center gap-3">
            <Button variant="secondary" onClick={greet}>
              调用 Rust
            </Button>
            <span className="text-sm text-muted-foreground">{greetMsg}</span>
          </div>
        </CardContent>
      </Card>
    </main>
  );
}

export default App;
