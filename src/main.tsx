import React from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/home";
import Test from "./pages/test";
import "./styles.css";
import {BrowserRouter, Routes, Route} from "react-router-dom";
import Config from "./pages/config.tsx";

document.addEventListener('DOMContentLoaded', function () {
    const currentTheme = localStorage.getItem('theme');

    if (currentTheme === null) {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            document.body.classList.add('dark');
            localStorage.setItem('theme', 'dark');
        }
        else {
            localStorage.setItem('theme', 'light');
        }
        return;
    }

    if (currentTheme !== 'light') {
        document.body.classList.add(currentTheme);
    }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
      <BrowserRouter>
          <Routes>
              <Route index element={<Home />} />
              <Route path="test" element={<Test />} />
              <Route path="config" element={<Config />} />
          </Routes>
      </BrowserRouter>
  </React.StrictMode>,
);
