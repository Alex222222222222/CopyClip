import "./App.css";
import { BrowserRouter, Outlet, Route, Routes } from "react-router-dom";
import Search from "./search/search";

function Layout() {
  return <>
    <Outlet />
  </>;
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route path="search" element={<Search />} />
          {
            // <Route index element={<Home />} />
            // <Route path="blogs" element={<Blogs />} />
            // <Route path="contact" element={<Contact />} />
            // <Route path="*" element={<NoPage />} />
          }
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
