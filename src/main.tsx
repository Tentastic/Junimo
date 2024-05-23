import React, {useState} from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/home";
import "./styles.css";
import {BrowserRouter, Routes, Route} from "react-router-dom";
import Config from "./pages/config.tsx";
import Profiles from "./pages/profiles.tsx";
import Exporter from "./pages/exporter.tsx";
import Splashscreen from "./pages/splashscreen.tsx";
import Importer from "./pages/importer.tsx";
import ModsProvider from "@components/ModsProvider.tsx";
import './i18n';
import SmapiPage from "./pages/smapi.tsx";
import Updater from "./pages/updater.tsx";


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
      <ModsProvider>
          <BrowserRouter>
              <Routes>
                  <Route index element={<Home />} />
                  <Route path="splashscreen" element={<Splashscreen />} />
                  <Route path="config" element={<Config />} />
                  <Route path="profiles" element={<Profiles />} />
                  <Route path="exporter" element={<Exporter />} />
                  <Route path="importer" element={<Importer />} />
                  <Route path="smapi" element={<SmapiPage />} />
                  <Route path="updater" element={<Updater />} />
              </Routes>
          </BrowserRouter>
      </ModsProvider>
  </React.StrictMode>,
);
