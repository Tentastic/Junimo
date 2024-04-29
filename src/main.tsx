import React from "react";
import ReactDOM from "react-dom/client";
import Home from "./pages/home";
import Test from "./pages/test";
import "./styles.css";
import {BrowserRouter, Routes, Route} from "react-router-dom";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
      <BrowserRouter>
          <Routes>
              <Route index element={<Home />} />
              <Route path="test" element={<Test />} />
          </Routes>
      </BrowserRouter>
  </React.StrictMode>,
);
