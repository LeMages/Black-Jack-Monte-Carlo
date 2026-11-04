/**
 * Structure commune : barre du haut, navigation latérale, contenu, pied de page.
 */

// Importation des modules framework
import { useEffect } from "react";
import { Outlet, useLocation } from "react-router-dom";

// Importation des modules internes
import FramingNote from "./FramingNote";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";

export default function AppLayout() {
  const { pathname } = useLocation();
  // Changement de page : retour en haut (un changement de config garde la position)
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [pathname]);
  return (
    <div className="app">
      <TopBar />
      <div className="app__body">
        <Sidebar />
        <main className="app__main">
          <div className="app__content">
            <Outlet />
            <FramingNote />
          </div>
        </main>
      </div>
    </div>
  );
}
