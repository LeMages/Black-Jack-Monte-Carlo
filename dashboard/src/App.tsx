/**
 * Composant racine : chargement des données et routes du dashboard.
 */

// Importation des modules framework
import { BrowserRouter, Route, Routes } from "react-router-dom";

// Importation des modules internes
import AppLayout from "./components/AppLayout";
import NotFound from "./components/NotFound";
import DataProvider from "./data/DataProvider";
import ComparisonView from "./views/comparison/ComparisonView";
import HomeView from "./views/home/HomeView";
import SystemView from "./views/system/SystemView";

export default function App() {
  return (
    <DataProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/" element={<HomeView />} />
            <Route path="/systeme/:slug" element={<SystemView />} />
            <Route path="/comparaison" element={<ComparisonView />} />
            <Route path="*" element={<NotFound />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </DataProvider>
  );
}
