import "./App.css";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import Search from "./pages/search";

function Layout() {
  return <></>;
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
