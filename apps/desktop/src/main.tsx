import "@celframe/ui/styles.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { initialTheme, useApp } from "./store";

// Before the first render, so the wrong theme never flashes.
useApp.getState().setTheme(initialTheme());

const root = document.getElementById("root");
if (!root) throw new Error("the page has no root element");
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
