import React from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/home";
import Test from "./pages/test";
import "./styles.css";
import {BrowserRouter, Routes, Route} from "react-router-dom";
import Config from "./pages/config.tsx";
import Profiles from "./pages/profiles.tsx";
import Exporter from "./pages/exporter.tsx";
import Splashscreen from "./pages/splashscreen.tsx";
import Importer from "./pages/importer.tsx";

document.addEventListener('DOMContentLoaded', function () {
    const currentTheme = localStorage.getItem('theme');

    if (currentTheme === null) {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            document.documentElement.classList.add('dark');
            localStorage.setItem('theme', 'dark');
        }
        else {
            localStorage.setItem('theme', 'light');
        }
        return;
    }

    if (currentTheme !== 'light') {
        document.documentElement.classList.add(currentTheme);
    }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
      <BrowserRouter>
          <Routes>
              <Route index element={<Home />} />
              <Route path="splashscreen" element={<Splashscreen />} />
              <Route path="test" element={<Test />} />
              <Route path="config" element={<Config />} />
              <Route path="profiles" element={<Profiles />} />
              <Route path="exporter" element={<Exporter />} />
              <Route path="importer" element={<Importer />} />
          </Routes>
      </BrowserRouter>
  </React.StrictMode>,
);
