import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("renders the project shell", () => {
    render(<App />);
    expect(screen.getByText("PikaXiangqi")).toBeInTheDocument();
  });

  it("increments the Zustand counter", () => {
    render(<App />);
    const button = screen.getByRole("button", { name: /计数 0/ });
    fireEvent.click(button);
    expect(screen.getByRole("button", { name: /计数 1/ })).toBeInTheDocument();
  });
});
